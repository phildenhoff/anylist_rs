//! Black-box versions of the three reproductions for issue #24 that used to
//! live in `src/realtime/sync.rs` (and reached into private fields). Same
//! intent, production timings on the paused clock.

use crate::realtime::fake::{Behaviour, FakeServer, Step};
use crate::realtime::support::*;
use anylist_rs::ConnectionState;
use tokio::time::Instant;

/// The client should send `--heartbeat--` on schedule even when the server
/// is quiet (reads everything, never sends text).
#[tokio::test(start_paused = true)]
async fn heartbeats_are_sent_while_server_is_silent() {
    guard(async {
        let server = FakeServer::start(Behaviour::silent());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(HB * 11 + TICK).await;
        let n = server.conn(0).heartbeats().len();
        assert_eq!(n, 11, "heartbeats in 11 intervals");
        assert_eq!(h.state().await, ConnectionState::Connected);
    })
    .await
}

/// A server that stops answering (no echo, no Pong) should be detected after
/// the liveness timeout and move to Reconnecting.
#[tokio::test(start_paused = true)]
async fn silent_server_is_detected_as_dead() {
    guard(async {
        let server = FakeServer::start(Behaviour::reject(503));
        server.script([Behaviour::raw()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();
        let seen = h
            .wait_state(ConnectionState::Reconnecting, LIVENESS + HB * 4)
            .await;
        let seen = seen.unwrap_or_else(|| panic!("not Reconnecting after {:?}", LIVENESS + HB * 4));
        assert_within(seen - t0, LIVENESS, LIVENESS + HB + TICK, "detected after");
    })
    .await
}

/// A server-initiated normal close (1000) should be recovered from, since
/// the client never asked to disconnect.
#[tokio::test(start_paused = true)]
async fn server_normal_close_triggers_reconnect() {
    guard(async {
        let server = FakeServer::start(Behaviour::reject(503));
        server.script([Behaviour::echo().steps(vec![Step::Close(1000)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(ms(100)).await;
        assert_eq!(h.state().await, ConnectionState::Reconnecting);
        assert!(server.attempts() >= 2, "no reconnect attempt");
    })
    .await
}
