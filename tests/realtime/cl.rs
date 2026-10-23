//! 2.5 Server close codes and stream end (CL)
//!
//! Unless stated, the fake's second connection accepts after a 50 ms
//! handshake delay and echoes heartbeats. The delay keeps the client in
//! `Reconnecting` for a while, so a poller can observe it.

use crate::fake::{Behaviour, FakeServer, RefreshReply, Step};
use crate::support::*;
use anylist_rs::{AuthEvent, ConnectionState, SyncEvent};
use tokio::time::Instant;

const HANDSHAKE: std::time::Duration = ms(50);

fn second() -> Behaviour {
    Behaviour::echo().delay_handshake(HANDSHAKE)
}

/// Server closes connection 0 with `code` after `before`; checks the common
/// "close is recovered" shape. Returns the server and harness.
async fn close_is_recovered(code: u16, before: Vec<Step>) -> (FakeServer, Harness, StatePoller) {
    let server = FakeServer::start(second());
    let mut steps = before;
    steps.push(Step::Close(code));
    server.script([Behaviour::echo().steps(steps)]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    let poller = h.poll_states(TICK);

    let c1 = server
        .wait_upgraded(1, MAX)
        .await
        .unwrap_or_else(|| panic!("no reconnect after close {code}"));
    // Let the poller sample the final state.
    tokio::time::sleep(ms(10)).await;

    let c0 = server.conn(0);
    let closed_at = c0.server_close_at.expect("fake never sent its close");
    let rec = poller
        .first_at_or_after(closed_at, ConnectionState::Reconnecting)
        .unwrap_or_else(|| panic!("Reconnecting never observed after close {code}"));
    assert_within(rec - closed_at, ZERO, TICK, "Reconnecting after the close");
    assert!(
        c1.accepted_at - closed_at <= INITIAL,
        "close {code}: second connection opened {:?} after the close",
        c1.accepted_at - closed_at
    );
    let up = c1.upgraded_at.unwrap();
    assert_eq!(up - c1.accepted_at, HANDSHAKE);
    let back = poller
        .first_at_or_after(up, ConnectionState::Connected)
        .expect("not Connected again");
    assert_within(back - up, ZERO, TICK, "Connected after the handshake");
    assert_eq!(
        poller.dedup(),
        vec![
            ConnectionState::Connected,
            ConnectionState::Reconnecting,
            ConnectionState::Connected
        ],
        "close {code}"
    );
    assert_eq!(
        h.events.reconnected(),
        1,
        "close {code}: {:?}",
        h.events.events()
    );
    (server, h, poller)
}

#[tokio::test(start_paused = true)]
async fn rt_cl_01_close_1000_from_the_server_is_recovered() {
    guard(async {
        let (server, h, _p) = close_is_recovered(
            1000,
            vec![
                Step::Sleep(ms(50)),
                Step::Text("refresh-shopping-lists".into()),
            ],
        )
        .await;
        assert_eq!(h.state().await, ConnectionState::Connected);
        let ev = h.events.events();
        assert_eq!(ev.len(), 2, "{ev:?}");
        assert_eq!(ev[0], SyncEvent::ShoppingListsChanged);
        assert!(is_reconnected(&ev[1]), "{ev:?}");
        assert_no_refresh(&server).await;
    })
    .await
}

/// Real time: the refresh is real HTTP. Ordering is checked causally; the
/// one time bound is the spec's `reconnect_initial_delay + 400 ms` (900 ms
/// here) against a few milliseconds of local work.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_02_close_4010_refreshes_tokens_then_reconnects_with_the_new_token() {
    guard_real(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(4010)])]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let (client, auth) = client_with_auth_log(&server.http_url());
        let h = Harness::new(client, real_config(&server));
        h.connect().await.unwrap();

        let c1 = server
            .wait_upgraded(1, secs(5))
            .await
            .expect("no reconnect after 4010");
        let c0 = server.conn(0);
        let closed_at = c0.server_close_at.unwrap();
        let up = c1.upgraded_at.unwrap();
        assert!(
            up - closed_at <= real::INITIAL + ms(400),
            "Connected again {:?} after the close",
            up - closed_at
        );
        assert!(h
            .wait_state(ConnectionState::Connected, secs(5))
            .await
            .is_some());
        assert!(eventually(secs(5), || async { h.events.reconnected() >= 1 }).await);

        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 1, "exactly one refresh");
        let r = &refreshes[0];
        assert_eq!(r.header("X-AnyLeaf-API-Version"), Some("3"));
        assert!(r.body.contains("name=\"refresh_token\"") && r.body.contains("refresh-1"));
        assert!(r.at > closed_at, "refresh before the close");
        assert!(r.at < c1.accepted_at, "refresh after the second connection");
        assert_eq!(c1.access_token().as_deref(), Some("access-2"));
        assert_eq!(h.events.reconnected(), 1);
        let auth = auth.lock().unwrap().clone();
        assert_eq!(auth, vec![AuthEvent::TokensRefreshed]);
        assert_eq!(server.refresh_connections(), 1);
    })
    .await
}

/// Real time: the refresh is real HTTP. Each refresh gap must be at least
/// its backoff delay (causal: a sleep never ends early) and less than
/// twice it, which tells doubling from a constant delay with a margin of
/// 500 ms or more.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_03_refresh_failure_after_4010_keeps_retrying_with_backoff() {
    guard_real(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(4010)])]);
        server.script_refresh([
            RefreshReply::Status(500),
            RefreshReply::Status(500),
            RefreshReply::Tokens("access-2".into(), "refresh-2".into()),
        ]);
        let (client, auth) = client_with_auth_log(&server.http_url());
        let h = Harness::new(client, real_config(&server));
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));

        let c1 = server
            .wait_upgraded(1, secs(10))
            .await
            .expect("no reconnect after the refresh succeeded");
        assert!(h
            .wait_state(ConnectionState::Connected, secs(5))
            .await
            .is_some());

        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 3, "three refresh requests");
        // No WebSocket attempt while refreshing: attempt 1 comes after the
        // third refresh.
        assert_eq!(server.connects(), 2);
        assert!(
            c1.accepted_at > refreshes[2].at,
            "connection attempted before the refresh succeeded"
        );
        let g = gaps(&refreshes.iter().map(|r| r.at).collect::<Vec<_>>());
        let d = real::INITIAL;
        assert!(g[0] >= d && g[0] < d * 2, "refresh gap 1: {:?}", g[0]);
        assert!(g[1] >= d * 2 && g[1] < d * 4, "refresh gap 2: {:?}", g[1]);
        let failed = auth
            .lock()
            .unwrap()
            .iter()
            .filter(|e| matches!(e, AuthEvent::RefreshFailed(_)))
            .count();
        assert_eq!(failed, 2);
        assert_eq!(c1.access_token().as_deref(), Some("access-2"));
        let closed_at = server.conn(0).server_close_at.unwrap();
        // Causal window: the client is Reconnecting before it sends the
        // first refresh, and until it opens the next connection.
        let gap = poller.between(refreshes[0].at, c1.accepted_at);
        assert!(!gap.is_empty());
        assert!(
            gap.iter().all(|s| *s == ConnectionState::Reconnecting),
            "state during the gap: {gap:?}"
        );
        assert!(refreshes[0].at > closed_at);
        assert!(!poller.ever(ConnectionState::Disconnected));
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cl_04_other_close_codes_reconnect_with_backoff() {
    guard(async {
        for code in [1001u16, 1011, 4000] {
            let (server, _h, _p) = close_is_recovered(code, vec![Step::Sleep(ms(50))]).await;
            assert_no_refresh(&server).await;
        }
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cl_05_tcp_drop_without_a_close_frame() {
    guard(async {
        let server = FakeServer::start(second());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(100)), Step::DropTcp])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();
        let poller = h.poll_states(TICK);

        let c1 = server
            .wait_upgraded(1, MAX)
            .await
            .expect("no reconnect after TCP drop");
        // Let the poller sample the final state.
        tokio::time::sleep(ms(10)).await;

        let dropped_at = server.conn(0).ended_at.unwrap();
        assert_eq!(dropped_at - t0, ms(100));
        let rec = poller
            .first_at_or_after(dropped_at, ConnectionState::Reconnecting)
            .expect("Reconnecting never observed");
        assert_within(rec - dropped_at, ZERO, TICK, "Reconnecting after drop");
        let up = c1.upgraded_at.unwrap();
        assert!(
            up - dropped_at <= INITIAL + HANDSHAKE,
            "Connected again {:?} after drop",
            up - dropped_at
        );
        let back = poller
            .first_at_or_after(up, ConnectionState::Connected)
            .expect("not Connected again");
        assert_within(back - up, ZERO, TICK, "Connected after the handshake");
        assert_eq!(h.events.reconnected(), 1);
        assert_no_refresh(&server).await;
    })
    .await
}

/// RT-CL-06 is a code-review requirement (a `None` from the stream ends the
/// session with `Reconnecting`, or `Closed` after our own close). `None` is
/// only reachable after a close handshake, so this covers both observable
/// halves: a server-completed close handshake followed by EOF is recovered,
/// and EOF after our own close does not reconnect.
#[tokio::test(start_paused = true)]
async fn rt_cl_06_stream_eof_is_treated_like_a_drop() {
    guard(async {
        let server = FakeServer::start(second());
        server.script([Behaviour::echo().steps(vec![Step::Close(1000)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(
            server.wait_upgraded(1, MAX).await.is_some(),
            "stream end not recovered"
        );
        settle().await;
        assert_eq!(h.state().await, ConnectionState::Connected);

        h.disconnect().await.unwrap();
        tokio::time::sleep(HB * 5).await;
        drain_real_io().await;
        assert_eq!(h.state().await, ConnectionState::Closed);
        assert_eq!(server.connects(), 2, "reconnected after our own close");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cl_07_close_1000_arriving_after_our_own_close_frame_does_not_reconnect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();
        let poller = h.poll_states(ms(10));
        drain_real_io().await;
        tokio::time::sleep(HB * 10).await;
        assert_eq!(poller.dedup(), vec![ConnectionState::Closed]);
        let c = server.conn(0);
        assert_eq!(c.close_frames(), vec![Some(1000)]);
        assert!(c.ended_at.is_some());
        assert_eq!(server.connects(), 1);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cl_08_server_closes_immediately_after_accepting_during_a_reconnect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let flap = || Behaviour::echo().steps(vec![Step::Close(1000)]);
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::Close(1000)]),
            flap(),
            flap(),
            flap(),
        ]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(TICK);

        assert!(
            server.wait_upgraded(4, MAX * 2).await.is_some(),
            "never reached connection 5"
        );
        settle().await;
        assert_eq!(h.state().await, ConnectionState::Connected);

        assert!(
            !poller.ever(ConnectionState::Disconnected),
            "{:?}",
            poller.dedup()
        );
        let accepts: Vec<Instant> = server.conns().iter().map(|c| c.accepted_at).collect();
        let g = gaps(&accepts);
        // The short sessions do not reset the backoff: it keeps doubling.
        assert_eq!(g[1], INITIAL, "accept gap 2->3");
        assert_eq!(g[2], INITIAL * 2, "accept gap 3->4");
        assert_eq!(g[3], INITIAL * 4, "accept gap 4->5");
        // Documented choice (D8): one Reconnected per successful handshake.
        assert_eq!(h.events.reconnected(), 4, "{:?}", h.events.events());
        assert_no_refresh(&server).await;
    })
    .await
}
