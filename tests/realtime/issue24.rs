//! Black-box versions of the three reproductions for issue #24 that used to
//! live in `src/realtime/sync.rs` (and reached into private fields). Same
//! intent, millisecond timings.

use crate::fake::{Behaviour, FakeServer, Step};
use crate::support::*;
use anylist_rs::ConnectionState;

/// The client should send `--heartbeat--` on schedule even when the server
/// is quiet (reads everything, never sends text).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn heartbeats_are_sent_while_server_is_silent() {
    guard(async {
        let server = FakeServer::start(Behaviour::silent()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(HB * 11).await;
        let n = server.conn(0).heartbeats().len();
        assert!(
            n >= 2,
            "expected at least 2 heartbeats in 11 intervals, server got {n}"
        );
    })
    .await
}

/// A server that stops answering (no echo, no Pong) should be detected after
/// the liveness timeout and move to Reconnecting.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn silent_server_is_detected_as_dead() {
    guard(async {
        let server = FakeServer::start(Behaviour::reject(503)).await;
        server.script([Behaviour::raw()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(
            h.wait_state(ConnectionState::Reconnecting, LIVENESS + HB * 4)
                .await
                .is_some(),
            "state is still {:?}",
            h.state().await
        );
    })
    .await
}

/// A server-initiated normal close (1000) should be recovered from, since
/// the client never asked to disconnect.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn server_normal_close_triggers_reconnect() {
    guard(async {
        let server = FakeServer::start(Behaviour::reject(503)).await;
        server.script([Behaviour::echo().steps(vec![Step::Close(1000)])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(ms(100)).await;
        assert_eq!(h.state().await, ConnectionState::Reconnecting);
    })
    .await
}
