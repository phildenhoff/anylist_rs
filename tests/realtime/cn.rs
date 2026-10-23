//! 2.1 Connecting (CN)

use crate::fake::{Behaviour, FakeServer, RefreshReply};
use crate::support::*;
use anylist_rs::{AnyListError, ConnectionState, SyncEvent};
use std::time::{Duration, Instant};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_01_successful_first_connect() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);

        let t = Instant::now();
        h.connect().await.expect("connect");
        assert!(t.elapsed() <= ms(500), "connect took {:?}", t.elapsed());
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert!(h.is_connected().await);

        tokio::time::sleep(ms(100)).await;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_02_first_attempt_refused_returns_err_and_leaves_disconnected() {
    guard(async {
        let port = free_port().await;
        let url = format!("ws://127.0.0.1:{port}");
        let h = Harness::new(
            client_at(&format!("http://127.0.0.1:{port}")),
            config_for_url(&url),
        );

        let t = Instant::now();
        let r = h.connect().await;
        assert!(t.elapsed() <= ms(500), "connect took {:?}", t.elapsed());
        assert!(
            matches!(r, Err(AnyListError::NetworkError(_))),
            "expected NetworkError, got {r:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Disconnected);

        // No actor left running: nothing connects once a server appears.
        let server = FakeServer::start_on(port, Behaviour::echo()).await;
        tokio::time::sleep(HB * 5).await;
        assert_eq!(
            server.attempts(),
            0,
            "no connection after a failed connect()"
        );
        assert!(h.disconnect().await.is_ok());
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_03_handshake_hang_is_bounded_by_connect_timeout() {
    guard(async {
        let server = FakeServer::start(Behaviour::hang()).await;
        let h = Harness::for_server(&server);

        let t = Instant::now();
        let r = tokio::time::timeout(ms(1500), h.connect()).await;
        let elapsed = t.elapsed();
        let r = r.expect("connect() did not return within 1.5 s against a hung handshake");
        assert!(
            matches!(r, Err(AnyListError::NetworkError(_))),
            "expected NetworkError, got {r:?}"
        );
        assert!(
            elapsed <= ms(700),
            "returned after {elapsed:?}, bound 700 ms"
        );
        assert!(
            elapsed >= ms(400),
            "returned after {elapsed:?}, before 400 ms"
        );
        assert_eq!(h.state().await, ConnectionState::Disconnected);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_04_connect_is_idempotent_while_connected() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        let t = Instant::now();
        h.connect().await.expect("second connect");
        assert!(
            t.elapsed() <= ms(50),
            "second connect took {:?}",
            t.elapsed()
        );
        assert_eq!(server.attempts(), 1);

        server.send_text(0, "refresh-shopping-lists");
        assert!(eventually(ms(500), || async { h.events.len() >= 1 }).await);
        tokio::time::sleep(ms(100)).await;
        assert_eq!(h.events.events(), vec![SyncEvent::ShoppingListsChanged]);
        assert_eq!(server.attempts(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_05_connect_after_closed_is_an_error() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();

        let t = Instant::now();
        let r = h.connect().await;
        assert!(t.elapsed() <= ms(50), "took {:?}", t.elapsed());
        assert!(matches!(r, Err(AnyListError::NetworkError(_))), "got {r:?}");
        assert_eq!(h.state().await, ConnectionState::Closed);
        tokio::time::sleep(HB * 5).await;
        assert_eq!(server.attempts(), 1, "no new connection after Closed");
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_06_retry_after_a_failed_first_connect() {
    guard(async {
        let port = free_port().await;
        let url = format!("ws://127.0.0.1:{port}");
        let h = Harness::new(
            client_at(&format!("http://127.0.0.1:{port}")),
            config_for_url(&url),
        );
        assert!(h.connect().await.is_err());

        let server = FakeServer::start_on(port, Behaviour::echo()).await;
        h.connect().await.expect("second connect");
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_eq!(server.upgraded().len(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_07_start_realtime_sync_propagates_the_first_failure() {
    guard(async {
        // No listener: Err. Default config, WebSocket URL derived from the
        // client's base URL.
        let port = free_port().await;
        let client = client_at(&format!("http://127.0.0.1:{port}"));
        let r = client.start_realtime_sync(|_| {}).await;
        assert!(r.is_err(), "expected Err with no listener");

        // A listening fake on the client's base URL: Ok and Connected.
        let server = FakeServer::start(Behaviour::echo()).await;
        let client = client_at(&server.http_url());
        let mut sync = client
            .start_realtime_sync(|_| {})
            .await
            .expect("start_realtime_sync");
        assert_eq!(sync.state().await, ConnectionState::Connected);
        assert_eq!(server.upgraded().len(), 1);
        assert_eq!(server.conn(0).path, "/data/add-user-listener");
        let _ = tokio::time::timeout(Duration::from_secs(2), sync.disconnect()).await;
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_08_handshake_401_on_first_connect_refreshes_once_and_retries() {
    guard(async {
        // Refresh succeeds.
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([Behaviour::reject(401)]);
        server.script_refresh([RefreshReply::Tokens("access-2".into(), "refresh-2".into())]);
        let h = Harness::for_server(&server);

        let t = Instant::now();
        let r = h.connect().await;
        assert!(t.elapsed() <= ms(1000), "took {:?}", t.elapsed());
        assert!(r.is_ok(), "connect: {r:?}");
        let refreshes = server.refreshes();
        assert_eq!(refreshes.len(), 1, "exactly one refresh");
        assert!(refreshes[0].body.contains("name=\"refresh_token\""));
        assert!(refreshes[0].body.contains("refresh-1"));
        let conns = server.conns();
        assert_eq!(conns.len(), 2);
        assert_eq!(conns[0].access_token().as_deref(), Some("access-1"));
        assert_eq!(conns[1].access_token().as_deref(), Some("access-2"));
        let tokens = h.client.export_tokens().unwrap();
        assert_eq!(tokens.access_token(), "access-2");
        assert_eq!(tokens.refresh_token(), "refresh-2");

        // Refresh fails.
        let server = FakeServer::start(Behaviour::echo()).await;
        server.script([Behaviour::reject(401)]);
        server.script_refresh([RefreshReply::Status(500)]);
        let h = Harness::for_server(&server);
        let r = tokio::time::timeout(ms(1000), h.connect())
            .await
            .expect("connect bounded");
        assert!(
            matches!(r, Err(AnyListError::AuthenticationFailed(_))),
            "expected AuthenticationFailed, got {r:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Disconnected);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_cn_09_cancelling_connect_abandons_the_attempt() {
    guard(async {
        let server = FakeServer::start(Behaviour::hang()).await;
        let h = Harness::for_server(&server);

        let r = tokio::time::timeout(ms(100), h.connect()).await;
        assert!(r.is_err(), "connect() should still be pending at 100 ms");
        assert!(
            h.wait_state(ConnectionState::Disconnected, ms(200))
                .await
                .is_some(),
            "state is {:?}, expected Disconnected after cancelling connect()",
            h.state().await
        );

        server.set_default(Behaviour::echo());
        tokio::time::sleep(HB * 5).await;
        assert_eq!(server.attempts(), 1, "no attempt after cancellation");

        h.connect().await.expect("later connect");
        assert_eq!(h.state().await, ConnectionState::Connected);
    })
    .await
}

/// Not a numbered scenario: section 3.3 validation.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_validation_rejects_bad_settings() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        for cfg in [
            config(&server).with_heartbeat_interval(Duration::ZERO),
            config(&server).with_liveness_timeout(Duration::ZERO),
            config(&server).with_reconnect_delays(ms(500), ms(100)),
        ] {
            let h = Harness::new(client_at(&server.http_url()), cfg);
            let r = h.connect().await;
            assert!(matches!(r, Err(AnyListError::Other(_))), "got {r:?}");
        }
        assert_eq!(server.attempts(), 0);
    })
    .await
}
