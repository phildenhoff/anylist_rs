//! 2.5 Server close codes and stream end (CL)
//!
//! Unless stated, the fake's second connection accepts after a 50 ms
//! handshake delay and echoes heartbeats. The delay keeps the client in
//! `Reconnecting` long enough for a 5 ms poller to observe it.

use crate::fake::{Behaviour, FakeServer, RefreshReply, Step};
use crate::support::*;
use anylist_rs::{AuthEvent, ConnectionState, SyncEvent};
use std::time::Instant;

fn second() -> Behaviour {
    Behaviour::echo().delay_handshake(ms(50))
}

/// Server closes connection 0 with `code` after `after`; checks the common
/// "close is recovered" shape. Returns the server and harness.
async fn close_is_recovered(code: u16, before: Vec<Step>) -> (FakeServer, Harness, StatePoller) {
    let server = FakeServer::start(second()).await;
    let mut steps = before;
    steps.push(Step::Close(code));
    server.script([Behaviour::echo().steps(steps)]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    let poller = h.poll_states(ms(5));

    let c1 = server
        .wait_upgraded(1, ms(1500))
        .await
        .unwrap_or_else(|| panic!("no reconnect after close {code}"));
    assert!(h
        .wait_state(ConnectionState::Connected, ms(300))
        .await
        .is_some());
    assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);
    tokio::time::sleep(ms(50)).await;

    let c0 = server.conn(0);
    let closed_at = c0.server_close_at.expect("fake never sent its close");
    let rec = poller
        .first_at_or_after(closed_at, ConnectionState::Reconnecting)
        .unwrap_or_else(|| panic!("Reconnecting never observed after close {code}"));
    assert!(
        rec - closed_at <= ms(100),
        "close {code}: Reconnecting observed {:?} after the close",
        rec - closed_at
    );
    assert!(
        c1.tcp_accepted_at - closed_at <= INITIAL + ms(200),
        "close {code}: second connection accepted {:?} after the close",
        c1.tcp_accepted_at - closed_at
    );
    assert!(
        !poller.ever(ConnectionState::Disconnected),
        "{:?}",
        poller.dedup()
    );
    assert_eq!(
        h.events.reconnected(),
        1,
        "close {code}: {:?}",
        h.events.events()
    );
    (server, h, poller)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_01_close_1000_from_the_server_is_recovered() {
    guard(async {
        let (_server, h, _p) = close_is_recovered(
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
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_02_close_4010_refreshes_tokens_then_reconnects_with_the_new_token() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(4010)])]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let (client, auth) = client_with_auth_log(&server.http_url());
        let h = Harness::new(client, config(&server));
        h.connect().await.unwrap();

        let c1 = server
            .wait_upgraded(1, ms(1500))
            .await
            .expect("no reconnect after 4010");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        let c0 = server.conn(0);
        let closed_at = c0.server_close_at.unwrap();
        assert!(
            c1.upgraded_at.unwrap() - closed_at <= INITIAL + ms(400),
            "Connected again {:?} after the close",
            c1.upgraded_at.unwrap() - closed_at
        );
        assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);
        tokio::time::sleep(ms(100)).await;

        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 1, "exactly one refresh");
        let r = &refreshes[0];
        assert_eq!(r.header("X-AnyLeaf-API-Version"), Some("3"));
        assert!(r.body.contains("name=\"refresh_token\"") && r.body.contains("refresh-1"));
        assert!(r.at > closed_at, "refresh before the close");
        assert!(r.at < c1.request_at, "refresh after the second upgrade");
        assert_eq!(c1.access_token().as_deref(), Some("access-2"));
        assert_eq!(h.events.reconnected(), 1);
        let auth = auth.lock().unwrap().clone();
        assert_eq!(auth, vec![AuthEvent::TokensRefreshed]);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_03_refresh_failure_after_4010_keeps_retrying_with_backoff() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(50)), Step::Close(4010)])]);
        server.script_refresh([
            RefreshReply::Status(500),
            RefreshReply::Status(500),
            RefreshReply::Tokens("access-2".into(), "refresh-2".into()),
        ]);
        let (client, auth) = client_with_auth_log(&server.http_url());
        let h = Harness::new(client, config(&server));
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));

        let c1 = server
            .wait_upgraded(1, ms(2500))
            .await
            .expect("no reconnect after the refresh succeeded");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        tokio::time::sleep(ms(50)).await;

        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 3, "three refresh requests");
        // No WebSocket attempt while refreshing: attempt 1 comes after the
        // third refresh.
        assert_eq!(server.attempts(), 2);
        assert!(
            c1.request_at > refreshes[2].at,
            "upgrade attempted before the refresh succeeded"
        );
        let g = gaps(&refreshes.iter().map(|r| r.at).collect::<Vec<_>>());
        assert_near(g[0], INITIAL, 50, "refresh gap 1");
        assert_near(g[1], INITIAL * 2, 50, "refresh gap 2");
        let failed = auth
            .lock()
            .unwrap()
            .iter()
            .filter(|e| matches!(e, AuthEvent::RefreshFailed(_)))
            .count();
        assert_eq!(failed, 2);
        assert_eq!(c1.access_token().as_deref(), Some("access-2"));
        let closed_at = server.conn(0).server_close_at.unwrap();
        let gap = poller.between(closed_at + ms(20), c1.request_at);
        assert!(!gap.is_empty());
        assert!(
            gap.iter().all(|s| *s == ConnectionState::Reconnecting),
            "state during the gap: {gap:?}"
        );
        assert!(!poller.ever(ConnectionState::Disconnected));
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_04_other_close_codes_reconnect_with_backoff() {
    guard(async {
        let runs = [1001u16, 1011, 4000].map(|code| async move {
            let (server, _h, _p) = close_is_recovered(code, vec![Step::Sleep(ms(50))]).await;
            assert_eq!(
                server.refreshes().len(),
                0,
                "close {code}: refresh requested"
            );
        });
        concurrently(runs.into()).await;
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_05_tcp_drop_without_a_close_frame() {
    guard(async {
        let server = FakeServer::start(second()).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(100)), Step::DropTcp])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));

        let c1 = server
            .wait_upgraded(1, ms(1500))
            .await
            .expect("no reconnect after TCP drop");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);
        tokio::time::sleep(ms(50)).await;

        let dropped_at = server.conn(0).ended_at.unwrap();
        let rec = poller
            .first_at_or_after(dropped_at, ConnectionState::Reconnecting)
            .expect("Reconnecting never observed");
        assert!(
            rec - dropped_at <= ms(100),
            "Reconnecting {:?} after drop",
            rec - dropped_at
        );
        assert!(
            c1.upgraded_at.unwrap() - dropped_at <= INITIAL + ms(200),
            "Connected again {:?} after drop",
            c1.upgraded_at.unwrap() - dropped_at
        );
        assert_eq!(h.events.reconnected(), 1);
        assert_eq!(server.refreshes().len(), 0);
    })
    .await
}

/// RT-CL-06 is a code-review requirement (a `None` from the stream ends the
/// session with `Reconnecting`, or `Closed` after our own close). `None` is
/// only reachable after a close handshake, so this covers both observable
/// halves: a server-completed close handshake followed by EOF is recovered,
/// and EOF after our own close does not reconnect.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_06_stream_eof_is_treated_like_a_drop() {
    guard(async {
        let server = FakeServer::start(second()).await;
        server.script([Behaviour::echo().steps(vec![Step::Close(1000)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(
            server.wait_upgraded(1, ms(1500)).await.is_some(),
            "stream end not recovered"
        );
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());

        h.disconnect().await.unwrap();
        tokio::time::sleep(HB * 5).await;
        assert_eq!(h.state().await, ConnectionState::Closed);
        assert_eq!(server.attempts(), 2, "reconnected after our own close");
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_07_close_1000_arriving_after_our_own_close_frame_does_not_reconnect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();
        let poller = h.poll_states(ms(5));
        tokio::time::sleep(HB * 10).await;
        assert_eq!(poller.dedup(), vec![ConnectionState::Closed]);
        assert_eq!(server.attempts(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cl_08_server_closes_immediately_after_accepting_during_a_reconnect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let flap = || Behaviour::echo().steps(vec![Step::Close(1000)]);
        server.script([
            Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::Close(1000)]),
            flap(),
            flap(),
            flap(),
        ]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(5));

        let c4 = server
            .wait_upgraded(4, ms(2500))
            .await
            .expect("never reached connection 5");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        tokio::time::sleep(ms(100)).await;
        let _ = c4;

        assert!(
            !poller.ever(ConnectionState::Disconnected),
            "{:?}",
            poller.dedup()
        );
        let accepts: Vec<Instant> = server.conns().iter().map(|c| c.tcp_accepted_at).collect();
        let g = gaps(&accepts);
        assert_near(g[1], INITIAL, 50, "accept gap 2->3");
        assert_near(g[2], INITIAL * 2, 50, "accept gap 3->4");
        assert_near(g[3], INITIAL * 4, 50, "accept gap 4->5");
        // Documented choice (D8): one Reconnected per successful handshake.
        assert_eq!(h.events.reconnected(), 4, "{:?}", h.events.events());
    })
    .await
}
