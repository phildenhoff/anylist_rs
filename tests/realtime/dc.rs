//! 2.7 Disconnect (DC)

use crate::fake::{Behaviour, FakeServer, Step};
use crate::support::*;
use anylist_rs::ConnectionState;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_01_disconnect_from_connected_sends_a_close_frame_and_is_bounded() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        let t = Instant::now();
        h.disconnect().await.expect("disconnect");
        assert!(t.elapsed() <= ms(200), "disconnect took {:?}", t.elapsed());
        assert_eq!(h.state().await, ConnectionState::Closed);

        let c = server
            .wait_ended(0, ms(500))
            .await
            .expect("connection never ended");
        assert_eq!(
            c.close_frames(),
            vec![Some(1000)],
            "close frames before EOF"
        );
    })
    .await
}

/// Connection 0 drops after 200 ms; the next attempts are rejected with 503.
/// Returns once three attempts have failed (the pending delay is then the
/// fourth backoff step).
async fn in_backoff(
    cfg: impl Fn(&FakeServer) -> anylist_rs::RealtimeConfig,
) -> (FakeServer, Harness) {
    let server = FakeServer::start(Behaviour::reject(503)).await;
    server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
    let h = Harness::new(client_at(&server.http_url()), cfg(&server));
    h.connect().await.unwrap();
    let c3 = server
        .wait_attempt(3, ms(2000))
        .await
        .expect("three failed reconnect attempts");
    tokio::time::sleep(ms(10).saturating_sub(c3.request_at.elapsed())).await;
    (server, h)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_02_disconnect_during_backoff_is_bounded() {
    guard(async {
        let (_server, h) = in_backoff(config).await;
        assert_eq!(h.state().await, ConnectionState::Reconnecting);
        let t = Instant::now();
        let r = tokio::time::timeout(ms(2000), h.disconnect()).await;
        let elapsed = t.elapsed();
        assert!(r.is_ok(), "disconnect() did not return within 2 s");
        assert!(r.unwrap().is_ok());
        assert!(elapsed <= ms(100), "disconnect took {elapsed:?}");
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_03_disconnect_during_an_in_flight_reconnect_attempt_is_bounded() {
    guard(async {
        let server = FakeServer::start(Behaviour::hang()).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(100)), Step::DropTcp])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let c1 = server
            .wait_attempt(1, ms(2000))
            .await
            .expect("no reconnect attempt");
        tokio::time::sleep(ms(100).saturating_sub(c1.tcp_accepted_at.elapsed())).await;

        let t = Instant::now();
        let r = tokio::time::timeout(ms(2000), h.disconnect()).await;
        let elapsed = t.elapsed();
        assert!(r.is_ok(), "disconnect() did not return within 2 s");
        assert!(elapsed <= ms(100), "disconnect took {elapsed:?}");
        assert_eq!(h.state().await, ConnectionState::Closed);
        let c1 = server
            .wait_ended(1, ms(100))
            .await
            .expect("in-flight attempt's TCP not closed within 100 ms");
        assert!(c1.ended_at.unwrap() - t <= elapsed + ms(100));
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_04_disconnect_when_the_server_ignores_the_close_frame_is_bounded() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().ignore_close()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t = Instant::now();
        let r = tokio::time::timeout(ms(2000), h.disconnect()).await;
        let elapsed = t.elapsed();
        assert!(r.expect("disconnect() unbounded").is_ok());
        assert!(
            elapsed <= CLOSE_TIMEOUT + ms(200),
            "disconnect took {elapsed:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_05_no_callbacks_after_disconnect_returns() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().steps(vec![Step::TextEvery(
            "refresh-shopping-lists".into(),
            ms(2),
        )]))
        .await;
        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);
        let mut sync = anylist_rs::RealtimeSync::with_config(
            client_at(&server.http_url()),
            config(&server),
            move |_| {
                count2.fetch_add(1, Ordering::SeqCst);
            },
        );
        sync.connect().await.unwrap();
        tokio::time::sleep(ms(100)).await;
        sync.disconnect().await.unwrap();
        let a = count.load(Ordering::SeqCst);
        assert!(a > 0, "no events before disconnect");
        tokio::time::sleep(HB * 10).await;
        assert_eq!(
            count.load(Ordering::SeqCst),
            a,
            "callback ran after disconnect()"
        );
        assert_eq!(sync.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_06_closed_is_terminal_and_makes_no_further_attempts() {
    guard(async {
        // Max delay shortened to 200 ms so that 10 x reconnect_max_delay fits
        // the suite's ~3 s per-test budget.
        let max = ms(200);
        let (server, h) = in_backoff(|s| config(s).with_reconnect_delays(INITIAL, max)).await;
        let _ = tokio::time::timeout(ms(2000), h.disconnect()).await;
        let attempts = server.attempts();
        server.set_default(Behaviour::echo());
        tokio::time::sleep(max * 10).await;
        assert_eq!(server.attempts(), attempts, "attempt after disconnect()");
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_07_disconnect_is_idempotent() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();
        let frames = server.wait_ended(0, ms(500)).await.unwrap().frames.len();
        let t = Instant::now();
        h.disconnect().await.expect("second disconnect");
        assert!(t.elapsed() <= ms(50));
        assert_eq!(h.state().await, ConnectionState::Closed);
        tokio::time::sleep(HB * 2).await;
        assert_eq!(server.attempts(), 1);
        assert_eq!(server.conn(0).frames.len(), frames);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_08_disconnect_on_a_never_connected_instance() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        let t = Instant::now();
        h.disconnect().await.expect("disconnect");
        assert!(t.elapsed() <= ms(50));
        assert_eq!(h.state().await, ConnectionState::Closed);
        assert!(h.connect().await.is_err());
        assert_eq!(server.attempts(), 0);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dc_09_disconnect_returns_ok_even_when_the_close_frame_cannot_be_sent() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        server.drop_tcp(0);
        let t = Instant::now();
        let r = tokio::time::timeout(ms(2000), h.disconnect()).await;
        let elapsed = t.elapsed();
        assert!(r.expect("disconnect() unbounded").is_ok());
        assert!(
            elapsed <= CLOSE_TIMEOUT + ms(200),
            "disconnect took {elapsed:?}"
        );
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}
