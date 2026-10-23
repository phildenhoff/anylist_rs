//! 2.7 Disconnect (DC)

use crate::fake::{Behaviour, FakeServer, Step};
use crate::support::*;
use anylist_rs::ConnectionState;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn rt_dc_01_disconnect_from_connected_sends_a_close_frame_and_is_bounded() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();

        let t = Instant::now();
        h.disconnect().await.expect("disconnect");
        // The fake completes the close handshake at once, so nothing waits.
        assert_eq!(t.elapsed(), ZERO, "disconnect waited on a timer");
        assert_eq!(h.state().await, ConnectionState::Closed);

        let c = server
            .wait_ended(0, TICK)
            .await
            .expect("connection never ended");
        assert_eq!(c.ended_at, Some(t));
        assert_eq!(
            c.close_frames(),
            vec![Some(1000)],
            "close frames before EOF"
        );
    })
    .await
}

/// The delay pending after three failed reconnect attempts: the fourth
/// backoff step (the first retry is immediate), 4 x initial = 8 s.
pub const PENDING: std::time::Duration = INITIAL.saturating_mul(4);

/// Connection 0 drops after 200 ms; the next attempts are rejected with 503.
/// Returns `into` after the third failed attempt, that is `into` into the
/// pending backoff delay.
pub async fn in_backoff(into: std::time::Duration) -> (FakeServer, Harness) {
    let server = FakeServer::start(Behaviour::reject(503));
    server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    let c3 = server
        .wait_attempt(3, MAX)
        .await
        .expect("three failed reconnect attempts");
    until(c3.accepted_at + into).await;
    assert_eq!(server.attempts(), 4, "attempt before the delay ran out");
    assert_eq!(h.state().await, ConnectionState::Reconnecting);
    (server, h)
}

#[tokio::test(start_paused = true)]
async fn rt_dc_02_disconnect_during_backoff_is_bounded() {
    guard(async {
        // 10 ms into the delay (as in the spec), and halfway through it.
        for into in [ms(10), PENDING / 2] {
            let (_server, h) = in_backoff(into).await;
            let t = Instant::now();
            let r = h.disconnect().await;
            assert!(r.is_ok());
            assert_eq!(
                t.elapsed(),
                ZERO,
                "{into:?} in: disconnect waited on a timer"
            );
            assert_eq!(h.state().await, ConnectionState::Closed);
        }
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_03_disconnect_during_an_in_flight_reconnect_attempt_is_bounded() {
    guard(async {
        let server = FakeServer::start(Behaviour::hang());
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(100)), Step::DropTcp])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let c1 = server
            .wait_request(1, MAX)
            .await
            .expect("no reconnect attempt");
        until(c1.accepted_at + ms(100)).await;
        assert_eq!(h.state().await, ConnectionState::Reconnecting);

        let t = Instant::now();
        let r = h.disconnect().await;
        assert!(r.is_ok());
        assert_eq!(t.elapsed(), ZERO, "disconnect waited on a timer");
        assert_eq!(h.state().await, ConnectionState::Closed);
        let c1 = server
            .wait_ended(1, TICK)
            .await
            .expect("in-flight attempt's connection not closed");
        assert_eq!(c1.client_closed_at, Some(t));
        assert_eq!(c1.ended_at, Some(t));
        assert_eq!(server.connects(), 2);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_04_disconnect_when_the_server_ignores_the_close_frame_is_bounded() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().ignore_close());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t = Instant::now();
        let r = h.disconnect().await;
        let elapsed = t.elapsed();
        assert!(r.is_ok());
        assert!(elapsed <= CLOSE_TIMEOUT, "disconnect took {elapsed:?}");
        assert_eq!(h.state().await, ConnectionState::Closed);
        let c = server.wait_ended(0, TICK).await.expect("never closed");
        assert_eq!(c.close_frames(), vec![Some(1000)]);
        assert!(c.ended_at.unwrap() - t <= CLOSE_TIMEOUT);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_05_no_callbacks_after_disconnect_returns() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().steps(vec![Step::TextEvery(
            "refresh-shopping-lists".into(),
            ms(2),
        )]));
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
        tokio::time::sleep(ms(100) + TICK).await;
        sync.disconnect().await.unwrap();
        let a = count.load(Ordering::SeqCst);
        assert_eq!(a, 51, "events before disconnect");
        tokio::time::sleep(HB * 10).await;
        drain_real_io().await;
        assert_eq!(
            count.load(Ordering::SeqCst),
            a,
            "callback ran after disconnect()"
        );
        assert_eq!(sync.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_06_closed_is_terminal_and_makes_no_further_attempts() {
    guard(async {
        // Halfway through the pending delay.
        let (server, h) = in_backoff(PENDING / 2).await;
        // The server accepts again from the moment of the close, so a
        // pending attempt that survived disconnect() would connect.
        let attempts = server.connects();
        server.set_default(Behaviour::echo());
        let _ = tokio::time::timeout(CLOSE_TIMEOUT, h.disconnect()).await;
        tokio::time::sleep(MAX * 10).await;
        drain_real_io().await;
        assert_eq!(server.connects(), attempts, "attempt after disconnect()");
        assert_eq!(h.state().await, ConnectionState::Closed);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_07_disconnect_is_idempotent() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        h.disconnect().await.unwrap();
        let frames = server.wait_ended(0, TICK).await.unwrap().frames.len();
        let t = Instant::now();
        h.disconnect().await.expect("second disconnect");
        assert_eq!(t.elapsed(), ZERO);
        assert_eq!(h.state().await, ConnectionState::Closed);
        drain_real_io().await;
        tokio::time::sleep(HB * 2).await;
        assert_eq!(server.connects(), 1);
        assert_eq!(server.conn(0).frames.len(), frames);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_08_disconnect_on_a_never_connected_instance() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        let t = Instant::now();
        h.disconnect().await.expect("disconnect");
        assert_eq!(t.elapsed(), ZERO);
        assert_eq!(h.state().await, ConnectionState::Closed);
        assert!(h.connect().await.is_err());
        assert_eq!(server.connects(), 0);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dc_09_disconnect_returns_ok_even_when_the_close_frame_cannot_be_sent() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        // The drop and disconnect() happen at the same instant: the client
        // has not noticed the drop when it starts closing.
        server.drop_tcp(0);
        let t = Instant::now();
        let r = h.disconnect().await;
        let elapsed = t.elapsed();
        assert!(r.is_ok());
        assert!(elapsed <= CLOSE_TIMEOUT, "disconnect took {elapsed:?}");
        assert_eq!(h.state().await, ConnectionState::Closed);
        assert_eq!(server.connects(), 1);
    })
    .await
}
