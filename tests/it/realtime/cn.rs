//! 2.1 Connecting (CN)

use crate::realtime::fake::{Behaviour, End, FakeServer, RefreshReply};
use crate::realtime::support::*;
use anylist_rs::{AnyListError, ConnectionState, SyncEvent};
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn rt_cn_01_successful_first_connect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);

        let t = Instant::now();
        h.connect().await.expect("connect");
        assert_eq!(t.elapsed(), ZERO, "connect() waited on a timer");
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert!(h.is_connected().await);

        tokio::time::sleep(HB * 5).await;
        drain_real_io().await;
        let conns = server.conns();
        assert_eq!(conns.len(), 1, "exactly one connection");
        let c = &conns[0];
        assert!(c.upgraded());
        assert_eq!(c.path, "/data/add-user-listener");
        assert_eq!(
            c.param("client_id").as_deref(),
            Some(h.client.client_identifier())
        );
        assert_eq!(c.access_token().as_deref(), Some("access-1"));
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cn_02_first_attempt_refused_returns_err_and_leaves_disconnected() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        server.refuse(true);
        let h = Harness::for_server(&server);

        let t = Instant::now();
        let r = h.connect().await;
        assert_eq!(t.elapsed(), ZERO, "connect() waited on a timer");
        assert!(
            matches!(r, Err(AnyListError::NetworkError(_))),
            "expected NetworkError, got {r:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Disconnected);
        assert_eq!(server.refused(), 1, "exactly one connection attempt");

        // No actor left running: nothing connects once a server appears.
        server.refuse(false);
        drain_real_io().await;
        tokio::time::sleep(HB * 5).await;
        assert_eq!(server.connects(), 1, "connection after a failed connect()");
        assert!(h.disconnect().await.is_ok());
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cn_03_handshake_hang_is_bounded_by_connect_timeout() {
    guard(async {
        let server = FakeServer::start(Behaviour::hang());
        let h = Harness::for_server(&server);

        let t = Instant::now();
        let r = h.connect().await;
        assert_eq!(
            t.elapsed(),
            CONNECT_TIMEOUT,
            "connect() returned early or late"
        );
        assert!(
            matches!(r, Err(AnyListError::NetworkError(_))),
            "expected NetworkError, got {r:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Disconnected);

        // The abandoned handshake's connection is closed when it times out.
        let c = server
            .wait_ended(0, TICK)
            .await
            .expect("attempt still open");
        assert_eq!(c.ended_at.unwrap() - t, CONNECT_TIMEOUT);
        assert_eq!(c.end, Some(End::Eof));
        assert_eq!(server.connects(), 1);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cn_04_connect_is_idempotent_while_connected() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        let t = Instant::now();
        h.connect().await.expect("second connect");
        assert_eq!(t.elapsed(), ZERO);
        assert_eq!(server.attempts(), 1);

        let sent = Instant::now();
        server.send_text(0, "refresh-shopping-lists");
        settle().await;
        let recs = h.events.all();
        assert_eq!(recs.len(), 1, "{:?}", h.events.events());
        assert_eq!(recs[0].event, SyncEvent::ShoppingListsChanged);
        assert_eq!(recs[0].at, sent, "event delivered late");

        tokio::time::sleep(HB * 5).await;
        assert_eq!(h.events.events(), vec![SyncEvent::ShoppingListsChanged]);
        assert_eq!(server.attempts(), 1);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cn_05_connect_after_closed_is_an_error() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();

        let t = Instant::now();
        let r = h.connect().await;
        assert_eq!(t.elapsed(), ZERO);
        assert!(matches!(r, Err(AnyListError::NetworkError(_))), "got {r:?}");
        drain_real_io().await;
        assert_eq!(h.state().await, ConnectionState::Closed);
        tokio::time::sleep(HB * 5).await;
        assert_eq!(server.connects(), 1, "new connection after Closed");
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cn_06_retry_after_a_failed_first_connect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        server.refuse(true);
        let h = Harness::for_server(&server);
        assert!(h.connect().await.is_err());
        assert_eq!(h.state().await, ConnectionState::Disconnected);

        server.refuse(false);
        h.connect().await.expect("second connect");
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_eq!(server.refused(), 1);
        assert_eq!(server.upgraded().len(), 1);
    })
    .await
}

/// Real time and real TCP: `start_realtime_sync` takes no config, so this
/// is the test of the default `connect_async` path with the WebSocket URL
/// derived from the client's base URL. Nothing here is timed.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_07_start_realtime_sync_propagates_the_first_failure() {
    guard_real(async {
        // No listener: Err.
        let port = free_port().await;
        let client = client_at(&format!("http://127.0.0.1:{port}"));
        let r = client.start_realtime_sync(|_| {}).await;
        assert!(r.is_err(), "expected Err with no listener");

        // A listening fake on the client's base URL: Ok and Connected.
        let server = FakeServer::start_tcp(Behaviour::echo()).await;
        let client = client_at(&server.tcp_http_url());
        let mut sync = client
            .start_realtime_sync(|_| {})
            .await
            .expect("start_realtime_sync");
        assert_eq!(sync.state().await, ConnectionState::Connected);
        assert_eq!(server.upgraded().len(), 1);
        assert_eq!(server.conn(0).path, "/data/add-user-listener");
        assert_eq!(server.conn(0).access_token().as_deref(), Some("access-1"));
        sync.disconnect().await.unwrap();
        assert_eq!(sync.state().await, ConnectionState::Closed);
    })
    .await
}

/// Real time: the refresh is real HTTP. The only time bound is the spec's
/// 1 s, against a few milliseconds of local work.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_08_handshake_401_on_first_connect_refreshes_once_and_retries() {
    guard_real(async {
        // Refresh succeeds.
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::reject(401)]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let h = Harness::real(&server);

        let t = Instant::now();
        let r = h.connect().await;
        assert!(t.elapsed() <= secs(1), "took {:?}", t.elapsed());
        assert!(r.is_ok(), "connect: {r:?}");
        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 1, "exactly one refresh");
        assert!(refreshes[0].body.contains("name=\"refresh_token\""));
        assert!(refreshes[0].body.contains("refresh-1"));
        let conns = server.conns();
        assert_eq!(conns.len(), 2);
        assert_eq!(conns[0].access_token().as_deref(), Some("access-1"));
        assert_eq!(conns[1].access_token().as_deref(), Some("access-2"));
        assert!(
            refreshes[0].at > conns[0].request_at(),
            "refresh before the 401"
        );
        assert!(
            refreshes[0].at < conns[1].accepted_at,
            "refresh after the retry"
        );
        let tokens = h.client.export_tokens().unwrap();
        assert_eq!(tokens.access_token(), "access-2");
        assert_eq!(tokens.refresh_token(), "refresh-2");

        // Refresh fails.
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::reject(401)]);
        server.script_refresh([RefreshReply::Status(500)]);
        let h = Harness::real(&server);
        let t = Instant::now();
        let r = h.connect().await;
        assert!(t.elapsed() <= secs(1), "took {:?}", t.elapsed());
        assert!(
            matches!(r, Err(AnyListError::AuthenticationFailed(_))),
            "expected AuthenticationFailed, got {r:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Disconnected);
        assert_eq!(server.refreshes().len(), 1);
        assert_eq!(server.attempts(), 1, "retried after the refresh failed");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_cn_09_cancelling_connect_abandons_the_attempt() {
    guard(async {
        let server = FakeServer::start(Behaviour::hang());
        let h = Harness::for_server(&server);

        let t = Instant::now();
        let r = tokio::time::timeout(ms(100), h.connect()).await;
        assert!(r.is_err(), "connect() should still be pending at 100 ms");
        assert_eq!(
            h.state().await,
            ConnectionState::Disconnected,
            "state right after cancelling connect()"
        );
        // The attempt's connection is dropped with the future.
        let c = server.conn(0);
        assert_eq!(c.client_closed_at, Some(t + ms(100)));
        settle().await;
        assert_eq!(server.conn(0).ended_at, Some(t + ms(100)));
        drain_real_io().await;

        server.set_default(Behaviour::echo());
        tokio::time::sleep(HB * 5).await;
        assert_eq!(server.connects(), 1, "attempt after cancellation");

        h.connect().await.expect("later connect");
        assert_eq!(h.state().await, ConnectionState::Connected);
    })
    .await
}

/// Not a numbered scenario: section 3.3 validation.
#[tokio::test(start_paused = true)]
async fn config_validation_rejects_bad_settings() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        for cfg in [
            config(&server).with_heartbeat_interval(ZERO),
            config(&server).with_liveness_timeout(ZERO),
            config(&server).with_reconnect_delays(ms(500), ms(100)),
        ] {
            let h = Harness::new(client_at(&server.http_url()), cfg);
            let r = h.connect().await;
            assert!(matches!(r, Err(AnyListError::Other(_))), "got {r:?}");
            assert_eq!(h.state().await, ConnectionState::Disconnected);
        }
        assert_eq!(server.connects(), 0);
    })
    .await
}
