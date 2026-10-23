//! 2.6 Reconnect (RC)

use crate::fake::{Behaviour, FakeServer, RefreshReply, Step};
use crate::support::*;
use anylist_rs::{AnyListError, ConnectionState, SyncEvent};
use std::time::Instant;

fn request_times(server: &FakeServer) -> Vec<Instant> {
    server.conns().iter().map(|c| c.request_at).collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_01_backoff_timing_doubles_and_caps() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
        server.script((0..5).map(|_| Behaviour::reject(503)));
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));

        let c6 = server
            .wait_upgraded(6, ms(2800))
            .await
            .expect("7th attempt never accepted");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);

        let g = gaps(&request_times(&server)[1..]);
        let want = [INITIAL, INITIAL * 2, MAX, MAX, MAX];
        for (i, (got, want)) in g.iter().zip(want).enumerate() {
            assert_near(
                *got,
                want,
                50,
                &format!("gap between attempts {} and {}", i + 2, i + 3),
            );
        }
        let dropped_at = server.conn(0).ended_at.unwrap();
        let during = poller.between(dropped_at + ms(20), c6.request_at);
        assert!(
            during.iter().all(|s| *s == ConnectionState::Reconnecting),
            "{:?}",
            poller.dedup()
        );
        assert_eq!(h.events.reconnected(), 1);
        assert_eq!(server.refreshes().len(), 0);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_02_backoff_resets_only_after_a_stable_session() {
    guard(async {
        // (a) Recovered session lasts >= liveness_timeout, then drops: the
        // backoff starts again from the beginning.
        let stable = async {
            let server = FakeServer::start(Behaviour::echo()).await;
            server.script([
                Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp]),
                Behaviour::reject(503),
                Behaviour::reject(503),
                Behaviour::echo().steps(vec![Step::Sleep(LIVENESS + ms(100)), Step::DropTcp]),
                Behaviour::reject(503),
            ]);
            let h = Harness::for_server(&server);
            h.connect().await.unwrap();
            assert!(
                server.wait_upgraded(5, ms(2500)).await.is_some(),
                "(a) never recovered twice"
            );
            let t = request_times(&server);
            assert_near(
                t[5] - t[4],
                INITIAL,
                50,
                "(a) first gap after a stable session",
            );
        };
        // (b) Recovered session dies at once: the backoff carries on.
        let unstable = async {
            let server = FakeServer::start(Behaviour::echo()).await;
            server.script([
                Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::Close(1000)]),
                Behaviour::echo().steps(vec![Step::Close(1000)]),
                Behaviour::reject(503),
            ]);
            let h = Harness::for_server(&server);
            h.connect().await.unwrap();
            assert!(
                server.wait_upgraded(3, ms(2500)).await.is_some(),
                "(b) never recovered"
            );
            let t = request_times(&server);
            assert_near(t[2] - t[1], INITIAL, 50, "(b) gap after a short session");
            assert_near(t[3] - t[2], INITIAL * 2, 50, "(b) next gap");
        };
        tokio::join!(stable, unstable);
    })
    .await
}

/// Connections 0..=3 close with 1000 after 3 x interval; connection 4 stays.
/// Connections 1..=4 accept after 20 ms, so "accept" is ordered after the
/// previous connection's end as the fake records it.
async fn flapping_setup() -> (FakeServer, Harness) {
    let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(20))).await;
    let closing = |delay: bool| {
        let b = Behaviour::echo().steps(vec![Step::Sleep(HB * 3), Step::Close(1000)]);
        if delay {
            b.delay_handshake(ms(20))
        } else {
            b
        }
    };
    server.script([closing(false), closing(true), closing(true), closing(true)]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    (server, h)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_03_no_duplicate_heartbeat_senders_after_reconnects() {
    guard(async {
        let (server, h) = flapping_setup().await;
        let c4 = server
            .wait_upgraded(4, ms(2000))
            .await
            .expect("never reached connection 5");
        let start = c4.upgraded_at.unwrap();
        tokio::time::sleep(ms(10)).await;
        let poller = h.poll_states(ms(10));
        let end = start + HB * 20;
        tokio::time::sleep(end.saturating_duration_since(Instant::now())).await;
        let samples = poller.samples();
        assert!(
            samples
                .iter()
                .all(|(_, s)| *s == ConnectionState::Connected),
            "{:?}",
            poller.dedup()
        );
        let beats: Vec<Instant> = server
            .conn(4)
            .heartbeats()
            .into_iter()
            .filter(|t| *t <= end)
            .collect();
        assert!(
            (15..=25).contains(&beats.len()),
            "{} heartbeats on the final connection in 1 s",
            beats.len()
        );
        for g in gaps(&beats) {
            assert!(g >= HB / 2, "two heartbeats {g:?} apart");
        }
        assert_eq!(server.attempts(), 5);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_04_at_most_one_live_connection_at_any_time() {
    guard(async {
        let (server, _h) = flapping_setup().await;
        assert!(server.wait_upgraded(4, ms(2000)).await.is_some());
        tokio::time::sleep(ms(100)).await;
        assert_eq!(server.max_concurrent_open(), 1);
        let conns = server.conns();
        for w in conns.windows(2) {
            let ended = w[0].ended_at.expect("previous connection never ended");
            assert!(
                ended < w[1].upgraded_at.unwrap(),
                "connection {} ended after connection {} was accepted",
                w[0].index,
                w[1].index
            );
        }
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_05_tokens_are_refreshed_only_on_4010_or_http_401() {
    guard(async {
        let ordinary = async {
            let server = FakeServer::start(Behaviour::echo()).await;
            server.script([
                Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(1000)]),
                Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(1011)]),
                Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::DropTcp]),
                Behaviour::reject(503),
            ]);
            let h = Harness::for_server(&server);
            h.connect().await.unwrap();
            assert!(
                server.wait_upgraded(4, ms(2500)).await.is_some(),
                "never recovered"
            );
            assert!(h
                .wait_state(ConnectionState::Connected, ms(300))
                .await
                .is_some());
            tokio::time::sleep(ms(50)).await;
            assert_eq!(
                server.refreshes().len(),
                0,
                "refresh during ordinary outages"
            );
            for c in server.conns() {
                assert_eq!(
                    c.access_token().as_deref(),
                    Some("access-1"),
                    "attempt {}",
                    c.index
                );
            }
        };
        let expired = async {
            let server = FakeServer::start(Behaviour::echo()).await;
            server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(4010)])]);
            server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
            let h = Harness::for_server(&server);
            h.connect().await.unwrap();
            assert!(server.wait_upgraded(1, ms(1500)).await.is_some());
            tokio::time::sleep(ms(100)).await;
            assert_eq!(
                server.refreshes().len(),
                1,
                "4010 should refresh exactly once"
            );
        };
        tokio::join!(ordinary, expired);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_06_upgrade_rejected_with_401_during_reconnect_refreshes_once() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::DropTcp]),
            Behaviour::reject(401),
        ]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        let c2 = server
            .wait_upgraded(2, ms(1500))
            .await
            .expect("never reconnected");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);
        tokio::time::sleep(ms(100)).await;

        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 1, "exactly one refresh");
        let c1 = server.conn(1);
        assert!(refreshes[0].at > c1.request_at, "refresh before the 401");
        assert!(
            refreshes[0].at < c2.request_at,
            "refresh after the next upgrade"
        );
        assert_eq!(c1.access_token().as_deref(), Some("access-1"));
        assert_eq!(c2.access_token().as_deref(), Some("access-2"));
        assert_eq!(h.events.reconnected(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_07_reconnected_is_delivered_after_each_reconnect_not_on_first_connect() {
    guard(async {
        let server = FakeServer::start(
            Behaviour::echo().steps(vec![Step::Text("refresh-shopping-lists".into())]),
        )
        .await;
        server.script([Behaviour::echo()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(HB * 5).await;
        assert_eq!(h.events.reconnected(), 0, "Reconnected on first connect");

        server.close(0, 1000);
        assert!(
            eventually(ms(1000), || async {
                h.events.count(|e| *e == SyncEvent::ShoppingListsChanged) >= 1
            })
            .await,
            "no event from the new connection: {:?}",
            h.events.events()
        );
        tokio::time::sleep(ms(100)).await;
        let recs = h.events.all();
        let names: Vec<_> = recs.iter().map(|r| r.event.clone()).collect();
        assert_eq!(recs.len(), 2, "{names:?}");
        assert!(
            is_reconnected(&recs[0].event),
            "Reconnected must come first: {names:?}"
        );
        assert_eq!(
            recs[0].state,
            Some(ConnectionState::Connected),
            "state at Reconnected"
        );
        assert_eq!(recs[1].event, SyncEvent::ShoppingListsChanged);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_08_reconnect_while_connected_forces_a_fresh_session() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(30))).await;
        server.script([Behaviour::echo()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));
        tokio::time::sleep(ms(20)).await;

        let t = Instant::now();
        let r = h.reconnect().await;
        assert!(t.elapsed() <= ms(50), "reconnect() took {:?}", t.elapsed());
        assert!(r.is_ok(), "{r:?}");

        let c1 = server
            .wait_attempt(1, ms(500))
            .await
            .expect("no second connection");
        assert!(
            c1.tcp_accepted_at - t <= ms(200),
            "second connection after {:?}",
            c1.tcp_accepted_at - t
        );
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);
        tokio::time::sleep(ms(50)).await;

        let closes = server.conn(0).close_frames();
        assert!(
            closes.iter().any(|c| matches!(c, Some(1000) | Some(1001))),
            "connection 0 saw close frames {closes:?}"
        );
        assert!(
            poller.ever(ConnectionState::Reconnecting),
            "{:?}",
            poller.dedup()
        );
        assert_eq!(h.events.reconnected(), 1);
        assert_eq!(server.refreshes().len(), 0);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_09_reconnect_during_backoff_skips_the_remaining_delay() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
        server.script((0..3).map(|_| Behaviour::reject(503)));
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        // Three failed attempts: the next delay is 400 ms.
        let c3 = server
            .wait_attempt(3, ms(2000))
            .await
            .expect("three failed attempts");
        let since = c3.request_at.elapsed();
        tokio::time::sleep(ms(50).saturating_sub(since)).await;
        assert_eq!(h.state().await, ConnectionState::Reconnecting);

        let t = Instant::now();
        h.reconnect().await.expect("reconnect");
        let c4 = server
            .wait_attempt(4, ms(600))
            .await
            .expect("no attempt after reconnect()");
        assert!(
            c4.request_at - t <= ms(100),
            "next attempt {:?} after reconnect()",
            c4.request_at - t
        );
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_10_reconnect_after_closed_is_an_error() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();
        let t = Instant::now();
        let r = h.reconnect().await;
        assert!(t.elapsed() <= ms(50));
        assert!(matches!(r, Err(AnyListError::NetworkError(_))), "{r:?}");
        tokio::time::sleep(HB * 5).await;
        assert_eq!(server.attempts(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_11_reconnect_from_disconnected_behaves_as_connect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        let r = tokio::time::timeout(ms(1000), h.reconnect())
            .await
            .expect("bounded");
        assert!(r.is_ok(), "{r:?}");
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_eq!(server.upgraded().len(), 1);

        let port = free_port().await;
        let url = format!("ws://127.0.0.1:{port}");
        let h = Harness::new(
            client_at(&format!("http://127.0.0.1:{port}")),
            config_for_url(&url),
        );
        let r = tokio::time::timeout(ms(1000), h.reconnect())
            .await
            .expect("bounded");
        assert!(r.is_err(), "expected Err with no server");
        assert_eq!(h.state().await, ConnectionState::Disconnected);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_12_state_sequence_seen_by_a_tail_style_poller() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(50))).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(100)), Step::Close(1000)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));
        assert!(
            server.wait_upgraded(1, ms(1500)).await.is_some(),
            "never reconnected"
        );
        tokio::time::sleep(ms(100)).await;
        assert_eq!(
            poller.dedup(),
            vec![
                ConnectionState::Connected,
                ConnectionState::Reconnecting,
                ConnectionState::Connected
            ]
        );
    })
    .await
}
