//! 2.6 Reconnect (RC)

use crate::realtime::fake::{Behaviour, FakeServer, RefreshReply, Step};
use crate::realtime::support::*;
use anylist_rs::{AnyListError, ConnectionState, SyncEvent};
use tokio::time::Instant;

fn accept_times(server: &FakeServer) -> Vec<Instant> {
    server.conns().iter().map(|c| c.accepted_at).collect()
}

#[tokio::test(start_paused = true)]
async fn rt_rc_01_backoff_timing_doubles_and_caps() {
    guard(async {
        // Connection 0 drops after 200 ms, the next 8 attempts are rejected
        // (each one observable), the 10th is accepted. With the production
        // backoff that reaches the 120 s cap twice.
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
        server.script((0..8).map(|_| Behaviour::reject(503)));
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(10));

        let c9 = server
            .wait_upgraded(9, MAX * 4)
            .await
            .expect("10th attempt never accepted");
        settle().await;
        assert_eq!(h.state().await, ConnectionState::Connected);

        let g = gaps(&accept_times(&server)[1..]);
        let want = [
            INITIAL,
            INITIAL * 2,
            INITIAL * 4,
            INITIAL * 8,
            INITIAL * 16,
            INITIAL * 32,
            MAX,
            MAX,
        ];
        assert_eq!(g, want, "gaps between attempts 2..=10");
        let dropped_at = server.conn(0).ended_at.unwrap();
        let during = poller.between(dropped_at + TICK, c9.accepted_at - TICK);
        assert!(during.len() > 30_000, "{} samples", during.len());
        assert!(
            during.iter().all(|s| *s == ConnectionState::Reconnecting),
            "{:?}",
            poller.dedup()
        );
        assert_eq!(h.events.reconnected(), 1);
        assert_no_refresh(&server).await;
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_02_backoff_resets_only_after_a_stable_session() {
    guard(async {
        // (a) The recovered session lasts longer than liveness_timeout, then
        // drops: the backoff starts again from the beginning.
        let server = FakeServer::start(Behaviour::echo());
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp]),
            Behaviour::reject(503),
            Behaviour::reject(503),
            Behaviour::echo().steps(vec![Step::Sleep(LIVENESS + secs(1)), Step::DropTcp]),
            Behaviour::reject(503),
        ]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(
            server.wait_upgraded(5, MAX).await.is_some(),
            "(a) never recovered twice"
        );
        let t = accept_times(&server);
        assert_eq!(
            t[2] - t[1],
            INITIAL,
            "(a) backoff before the stable session"
        );
        assert_eq!(
            t[3] - t[2],
            INITIAL * 2,
            "(a) backoff before the stable session"
        );
        assert_eq!(t[5] - t[4], INITIAL, "(a) first gap after a stable session");
        drop(h);

        // (b) The recovered session dies at once: the backoff carries on.
        let server = FakeServer::start(Behaviour::echo());
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::Close(1000)]),
            Behaviour::echo().steps(vec![Step::Close(1000)]),
            Behaviour::reject(503),
        ]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(
            server.wait_upgraded(3, MAX).await.is_some(),
            "(b) never recovered"
        );
        let t = accept_times(&server);
        assert_eq!(t[2] - t[1], INITIAL, "(b) gap after a short session");
        assert_eq!(t[3] - t[2], INITIAL * 2, "(b) next gap");
    })
    .await
}

/// Connections 0..=3 close with 1000 after 3 x interval; connection 4 stays.
async fn flapping_setup() -> (FakeServer, Harness) {
    let server = FakeServer::start(Behaviour::echo());
    let closing = || Behaviour::echo().steps(vec![Step::Sleep(HB * 3), Step::Close(1000)]);
    server.script([closing(), closing(), closing(), closing()]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    (server, h)
}

#[tokio::test(start_paused = true)]
async fn rt_rc_03_no_duplicate_heartbeat_senders_after_reconnects() {
    guard(async {
        let (server, h) = flapping_setup().await;
        let c4 = server
            .wait_upgraded(4, MAX)
            .await
            .expect("never reached connection 5");
        let start = c4.upgraded_at.unwrap();
        until(start + TICK).await;
        let poller = h.poll_states(ms(10));
        let end = start + HB * 20 + TICK;
        until(end).await;
        assert_eq!(
            poller.dedup(),
            vec![ConnectionState::Connected],
            "left Connected on the final connection"
        );
        // Exactly one heartbeat per interval: t + k x interval.
        let beats: Vec<_> = server
            .conn(4)
            .heartbeats()
            .into_iter()
            .map(|t| t - start)
            .collect();
        let want: Vec<_> = (1..=20).map(|k| HB * k).collect();
        assert_eq!(beats, want, "heartbeats on the final connection");
        assert_eq!(server.connects(), 5);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_04_at_most_one_live_connection_at_any_time() {
    guard(async {
        let (server, _h) = flapping_setup().await;
        assert!(server.wait_upgraded(4, MAX).await.is_some());
        tokio::time::sleep(HB * 2).await;
        assert_eq!(server.max_concurrent_open(), 1);
        let conns = server.conns();
        assert_eq!(conns.len(), 5);
        for w in conns.windows(2) {
            let closed = w[0]
                .client_closed_seq
                .unwrap_or_else(|| panic!("connection {} never closed", w[0].index));
            assert!(
                closed < w[1].accepted_seq,
                "connection {} opened before connection {} was closed",
                w[1].index,
                w[0].index
            );
            let ended = w[0].ended_at.expect("previous connection never ended");
            assert!(
                ended <= w[1].accepted_at,
                "connection {} ended after connection {} was opened",
                w[0].index,
                w[1].index
            );
        }
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_05_tokens_are_not_refreshed_on_ordinary_outages() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(1000)]),
            Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(1011)]),
            Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::DropTcp]),
            Behaviour::reject(503),
        ]);
        let (client, auth) = client_with_auth_log(&server.http_url());
        let h = Harness::new(client, config(&server));
        h.connect().await.unwrap();
        assert!(
            server.wait_upgraded(4, MAX).await.is_some(),
            "never recovered"
        );
        settle().await;
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_no_refresh(&server).await;
        assert_eq!(auth.lock().unwrap().clone(), vec![]);
        for c in server.conns() {
            assert_eq!(
                c.access_token().as_deref(),
                Some("access-1"),
                "attempt {}",
                c.index
            );
        }
    })
    .await
}

/// Real time: the refresh is real HTTP. Nothing here is timed.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_05_tokens_are_refreshed_exactly_once_on_4010() {
    guard_real(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(4010)])]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let h = Harness::real(&server);
        h.connect().await.unwrap();
        let c1 = server
            .wait_upgraded(1, secs(5))
            .await
            .expect("no reconnect");
        assert!(h
            .wait_state(ConnectionState::Connected, secs(5))
            .await
            .is_some());
        // Nothing further happens.
        tokio::time::sleep(ms(200)).await;
        assert_eq!(
            server.refreshes().len(),
            1,
            "4010 should refresh exactly once"
        );
        assert_eq!(server.refresh_connections(), 1);
        assert_eq!(c1.access_token().as_deref(), Some("access-2"));
        assert_eq!(server.connects(), 2);
    })
    .await
}

/// Real time: the refresh is real HTTP. Ordering is checked causally.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_rc_06_upgrade_rejected_with_401_during_reconnect_refreshes_once() {
    guard_real(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::DropTcp]),
            Behaviour::reject(401),
        ]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let h = Harness::real(&server);
        h.connect().await.unwrap();

        let c2 = server
            .wait_upgraded(2, secs(5))
            .await
            .expect("never reconnected");
        assert!(h
            .wait_state(ConnectionState::Connected, secs(5))
            .await
            .is_some());
        assert!(eventually(secs(5), || async { h.events.reconnected() >= 1 }).await);
        // Nothing further happens.
        tokio::time::sleep(ms(200)).await;

        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 1, "exactly one refresh");
        assert_eq!(server.refresh_connections(), 1);
        let c1 = server.conn(1);
        assert!(refreshes[0].at > c1.request_at(), "refresh before the 401");
        assert!(
            refreshes[0].at < c2.accepted_at,
            "refresh after the next connection"
        );
        assert_eq!(c1.access_token().as_deref(), Some("access-1"));
        assert_eq!(c2.access_token().as_deref(), Some("access-2"));
        assert_eq!(h.events.reconnected(), 1);
        assert_eq!(server.connects(), 3);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_07_reconnected_is_delivered_after_each_reconnect_not_on_first_connect() {
    guard(async {
        let server = FakeServer::start(
            Behaviour::echo().steps(vec![Step::Text("refresh-shopping-lists".into())]),
        );
        server.script([Behaviour::echo()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(HB * 5).await;
        assert_eq!(h.events.reconnected(), 0, "Reconnected on first connect");
        assert_eq!(h.events.len(), 0);

        server.close(0, 1000);
        let c1 = server.wait_upgraded(1, MAX).await.expect("no reconnect");
        settle().await;
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
        let up = c1.upgraded_at.unwrap();
        assert_eq!((recs[0].at, recs[1].at), (up, up), "delivered late");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_08_reconnect_while_connected_forces_a_fresh_session() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(30)));
        server.script([Behaviour::echo()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(TICK);
        tokio::time::sleep(ms(20)).await;

        let t = Instant::now();
        let r = h.reconnect().await;
        assert_eq!(t.elapsed(), ZERO, "reconnect() waited on a timer");
        assert!(r.is_ok(), "{r:?}");

        let c1 = server
            .wait_upgraded(1, MAX)
            .await
            .expect("no second connection");
        assert_eq!(c1.accepted_at, t, "second connection delayed");
        // Let the poller sample the final state.
        tokio::time::sleep(ms(10)).await;
        assert_eq!(h.state().await, ConnectionState::Connected);

        let c0 = server.conn(0);
        let closes = c0.close_frames();
        assert!(
            closes.iter().any(|c| matches!(c, Some(1000) | Some(1001))),
            "connection 0 saw close frames {closes:?}"
        );
        assert!(c0.client_closed_seq.unwrap() < c1.accepted_seq);
        assert_eq!(
            poller.dedup(),
            vec![
                ConnectionState::Connected,
                ConnectionState::Reconnecting,
                ConnectionState::Connected
            ]
        );
        assert_eq!(h.events.reconnected(), 1);
        assert_no_refresh(&server).await;
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_09_reconnect_during_backoff_skips_the_remaining_delay() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
        server.script((0..3).map(|_| Behaviour::reject(503)));
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        // Three failed attempts: the next delay is 4 x initial (8 s).
        let c3 = server
            .wait_attempt(3, MAX)
            .await
            .expect("three failed attempts");
        until(c3.accepted_at + ms(50)).await;
        assert_eq!(h.state().await, ConnectionState::Reconnecting);
        assert_eq!(server.attempts(), 4, "attempt before the delay ran out");

        let t = Instant::now();
        h.reconnect().await.expect("reconnect");
        let c4 = server
            .wait_upgraded(4, INITIAL * 4)
            .await
            .expect("no attempt after reconnect()");
        assert_eq!(c4.accepted_at, t, "next attempt delayed");
        settle().await;
        assert_eq!(h.state().await, ConnectionState::Connected);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_10_reconnect_after_closed_is_an_error() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();
        let t = Instant::now();
        let r = h.reconnect().await;
        assert_eq!(t.elapsed(), ZERO);
        assert!(matches!(r, Err(AnyListError::NetworkError(_))), "{r:?}");
        tokio::time::sleep(HB * 5).await;
        drain_real_io().await;
        assert_eq!(server.connects(), 1);
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_11_reconnect_from_disconnected_behaves_as_connect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        let t = Instant::now();
        let r = h.reconnect().await;
        assert_eq!(t.elapsed(), ZERO);
        assert!(r.is_ok(), "{r:?}");
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_eq!(server.upgraded().len(), 1);
        assert_eq!(server.connects(), 1);

        let server = FakeServer::start(Behaviour::echo());
        server.refuse(true);
        let h = Harness::for_server(&server);
        let r = h.reconnect().await;
        assert!(r.is_err(), "expected Err with no server");
        assert_eq!(h.state().await, ConnectionState::Disconnected);
        assert_eq!(server.refused(), 1);
        server.refuse(false);
        drain_real_io().await;
        tokio::time::sleep(HB * 5).await;
        assert_eq!(
            server.connects(),
            1,
            "connection after a failed reconnect()"
        );
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_rc_12_state_sequence_seen_by_a_tail_style_poller() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(50)));
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(100)), Step::Close(1000)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));
        assert!(
            server.wait_upgraded(1, MAX).await.is_some(),
            "never reconnected"
        );
        tokio::time::sleep(HB * 4).await;
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
