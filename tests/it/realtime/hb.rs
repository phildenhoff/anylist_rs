//! 2.2 Heartbeats (HB)

use crate::realtime::fake::{Behaviour, FakeServer, Frame, Step, HEARTBEAT};
use crate::realtime::support::*;
use anylist_rs::{ConnectionState, SyncEvent};
use tokio::time::Instant;

/// Connect, then run for `n` heartbeat intervals (and one tick, so the
/// n-th heartbeat is in). Returns the time `connect()` returned.
async fn heartbeat_run(behaviour: Behaviour, n: u32) -> (FakeServer, Harness, Instant) {
    let server = FakeServer::start(behaviour);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    let t0 = Instant::now();
    tokio::time::sleep(HB * n + TICK).await;
    (server, h, t0)
}

/// Heartbeats exactly every interval after connecting: t0 + k x interval.
#[track_caller]
fn assert_schedule(beats: &[Instant], t0: Instant, n: u32) {
    let want: Vec<Instant> = (1..=n).map(|k| t0 + HB * k).collect();
    assert_eq!(
        beats.iter().map(|b| *b - t0).collect::<Vec<_>>(),
        want.iter().map(|b| *b - t0).collect::<Vec<_>>(),
        "heartbeat times (since connect)"
    );
}

#[tokio::test(start_paused = true)]
async fn rt_hb_01_heartbeats_are_sent_on_schedule_while_the_server_is_silent() {
    guard(async {
        let (server, _h, t0) = heartbeat_run(Behaviour::echo(), 20).await;
        assert_schedule(&server.conn(0).heartbeats(), t0, 20);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_hb_02_heartbeat_frame_content() {
    guard(async {
        let (server, _h, _) = heartbeat_run(Behaviour::echo(), 20).await;
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
        assert_eq!(texts, 20, "one heartbeat text frame per interval");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_hb_03_a_ping_accompanies_each_heartbeat() {
    guard(async {
        let (server, _h, t0) = heartbeat_run(Behaviour::echo(), 20).await;
        let c = server.conn(0);
        assert_schedule(&c.heartbeats(), t0, 20);
        // Exactly one Ping per heartbeat, at the same instant.
        assert_eq!(
            c.pings(),
            c.heartbeats(),
            "Ping times differ from heartbeats"
        );
        // And each Ping directly follows its heartbeat.
        let kinds: Vec<&Frame> = c
            .frames
            .iter()
            .map(|(_, f)| f)
            .filter(|f| matches!(f, Frame::Text(_) | Frame::Ping))
            .collect();
        for pair in kinds.chunks(2) {
            assert_eq!(
                pair,
                [&Frame::Text(HEARTBEAT.into()), &Frame::Ping],
                "frames {kinds:?}"
            );
        }
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_hb_04_heartbeats_keep_their_schedule_while_the_server_is_chatty() {
    guard(async {
        let b = Behaviour::echo().steps(vec![Step::TextEvery(
            "refresh-shopping-lists".into(),
            ms(10),
        )]);
        let (server, h, t0) = heartbeat_run(b, 20).await;
        assert_schedule(&server.conn(0).heartbeats(), t0, 20);
        // One event at t0 and every 10 ms up to and including t0 + 100 s.
        let n = h.events.count(|e| *e == SyncEvent::ShoppingListsChanged);
        let want = (HB * 20).as_millis() / 10 + 1;
        assert_eq!(n as u128, want, "ShoppingListsChanged delivered");
        assert!(h
            .events
            .all()
            .iter()
            .all(|r| (r.at - t0).as_millis() % 10 == 0));
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_hb_05_echoed_heartbeats_are_not_delivered_to_the_callback() {
    guard(async {
        let mut steps: Vec<Step> = (0..5).map(|_| Step::Text(HEARTBEAT.into())).collect();
        steps.push(Step::Text("refresh-user-recipe-data".into()));
        let server = FakeServer::start(Behaviour::echo().steps(steps));
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();
        settle().await;
        let recs = h.events.all();
        assert_eq!(h.events.events(), vec![SyncEvent::RecipeDataChanged]);
        assert_eq!(recs[0].at, t0, "delivered late");
        tokio::time::sleep(HB * 4).await;
        assert_eq!(h.events.events(), vec![SyncEvent::RecipeDataChanged]);
        assert_eq!(h.events.count(|e| *e == SyncEvent::Heartbeat), 0);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_hb_06_heartbeat_send_failure_ends_the_session() {
    guard(async {
        // Connection 0 keeps the session "alive" by sending a heartbeat every
        // interval, but every write the client makes on it fails. The next
        // connection is normal, with a 50 ms handshake.
        let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(50)));
        server.script([Behaviour::silent().steps(vec![
            Step::BreakClientWrites,
            Step::TextEvery(HEARTBEAT.into(), HB),
        ])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();

        let seen = h
            .wait_state(ConnectionState::Reconnecting, HB * 3 + ms(50))
            .await
            .expect("not Reconnecting within 3 intervals");
        // The first heartbeat write, one interval in, fails and ends the
        // session at once.
        assert_within(seen - t0, HB, HB + TICK, "Reconnecting after connect");
        assert_eq!(server.conn(0).client_closed_at, Some(t0 + HB));

        let c1 = server
            .wait_upgraded(1, secs(1))
            .await
            .expect("no reconnect after the write failure");
        assert_eq!(c1.accepted_at, t0 + HB, "reconnect attempt delayed");
        let back = h
            .wait_state(ConnectionState::Connected, secs(1))
            .await
            .expect("not Connected again");
        assert_within(back - c1.upgraded_at.unwrap(), ZERO, TICK, "Connected");
        assert_eq!(h.events.reconnected(), 1);
    })
    .await
}
