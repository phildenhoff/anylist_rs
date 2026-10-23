//! 2.2 Heartbeats (HB)

use crate::fake::{Behaviour, FakeServer, Frame, Step, HEARTBEAT};
use crate::support::*;
use anylist_rs::{ConnectionState, SyncEvent};
use std::time::Instant;

/// Connect to an echoing server and collect connection 0 after `secs`.
async fn heartbeat_run(behaviour: Behaviour, run: std::time::Duration) -> (FakeServer, Harness) {
    let server = FakeServer::start(behaviour).await;
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    tokio::time::sleep(run).await;
    (server, h)
}

#[track_caller]
fn assert_schedule(beats: &[Instant], min: usize) {
    assert!(
        beats.len() >= min,
        "expected at least {min} heartbeats, server got {}",
        beats.len()
    );
    for g in gaps(beats) {
        assert!(g <= HB * 3, "heartbeat gap {g:?} exceeds 3 x interval");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_hb_01_heartbeats_are_sent_on_schedule_while_the_server_is_silent() {
    guard(async {
        let (server, _h) = heartbeat_run(Behaviour::echo(), HB * 20).await;
        assert_schedule(&server.conn(0).heartbeats(), 10);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_hb_02_heartbeat_frame_content() {
    guard(async {
        let (server, _h) = heartbeat_run(Behaviour::echo(), HB * 20).await;
        let c = server.conn(0);
        let mut texts = 0;
        for (_, f) in &c.frames {
            match f {
                Frame::Text(t) => {
                    assert_eq!(
                        t.as_bytes(),
                        b"--heartbeat--",
                        "unexpected text frame {t:?}"
                    );
                    assert_eq!(t.len(), 13);
                    texts += 1;
                }
                Frame::Binary(n) => panic!("unexpected binary frame of {n} bytes"),
                _ => {}
            }
        }
        assert!(
            texts >= 10,
            "expected at least 10 heartbeat text frames, got {texts}"
        );
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_hb_03_a_ping_accompanies_each_heartbeat() {
    guard(async {
        let (server, _h) = heartbeat_run(Behaviour::echo(), HB * 20).await;
        let snapshot_at = Instant::now();
        let c = server.conn(0);
        let beats = c.heartbeats();
        let pings = c.pings();
        assert!(beats.len() >= 10, "only {} heartbeats", beats.len());
        for b in beats.iter().filter(|b| snapshot_at - **b > ms(20)) {
            let near = pings.iter().any(|p| {
                let d = if p > b { *p - *b } else { *b - *p };
                d <= ms(20)
            });
            assert!(
                near,
                "no Ping within 20 ms of a heartbeat ({} pings total)",
                pings.len()
            );
        }
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_hb_04_heartbeats_keep_their_schedule_while_the_server_is_chatty() {
    guard(async {
        let b = Behaviour::echo().steps(vec![Step::TextEvery(
            "refresh-shopping-lists".into(),
            ms(10),
        )]);
        let (server, h) = heartbeat_run(b, ms(1000)).await;
        assert_schedule(&server.conn(0).heartbeats(), 10);
        let n = h.events.count(|e| *e == SyncEvent::ShoppingListsChanged);
        assert!(n >= 50, "callback got only {n} ShoppingListsChanged");
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_hb_05_echoed_heartbeats_are_not_delivered_to_the_callback() {
    guard(async {
        let mut steps: Vec<Step> = (0..5).map(|_| Step::Text(HEARTBEAT.into())).collect();
        steps.push(Step::Text("refresh-user-recipe-data".into()));
        let server = FakeServer::start(Behaviour::echo().steps(steps)).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(eventually(ms(200), || async { h.events.len() >= 1 }).await);
        tokio::time::sleep(ms(100)).await;
        assert_eq!(h.events.events(), vec![SyncEvent::RecipeDataChanged]);
        assert_eq!(h.events.count(|e| *e == SyncEvent::Heartbeat), 0);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_hb_06_heartbeat_send_failure_ends_the_session() {
    guard(async {
        // Connection 0 keeps itself "alive" by sending heartbeats, but drops
        // TCP as soon as the client writes anything.
        let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(50))).await;
        server.script([Behaviour::silent()
            .drop_on_inbound()
            .steps(vec![Step::TextEvery(HEARTBEAT.into(), HB)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let seen = h
            .wait_state(ConnectionState::Reconnecting, HB * 3 + ms(50))
            .await;
        assert!(
            seen.is_some(),
            "state did not become Reconnecting within {:?} (now {:?})",
            HB * 3 + ms(50),
            h.state().await
        );
        assert!(
            server.wait_upgraded(1, ms(1000)).await.is_some(),
            "no reconnect after the write failure"
        );
        assert!(h
            .wait_state(ConnectionState::Connected, ms(500))
            .await
            .is_some());
    })
    .await
}
