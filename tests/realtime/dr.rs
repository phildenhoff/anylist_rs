//! 2.8 Drop (DR)

use crate::fake::{Behaviour, FakeServer, Step};
use crate::support::*;
use anylist_rs::RealtimeSync;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn rt_dr_01_drop_while_connected_ends_the_connection() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let sync = h.into_sync().await;
        let t = Instant::now();
        drop(sync);
        // On this single-threaded runtime nothing else has run yet, so a
        // drop that waited for the background task could not have returned:
        // the connection is still open.
        assert_eq!(t.elapsed(), ZERO);
        assert_eq!(server.conn(0).client_closed_at, None, "drop closed inline");
        let c = server
            .wait_ended(0, CLOSE_TIMEOUT)
            .await
            .expect("server did not see the connection end within close_timeout");
        assert_eq!(c.ended_at, Some(t), "connection ended late");
        // SHOULD: a close frame rather than a bare EOF.
        let closes = c.close_frames();
        assert!(
            closes.iter().any(|c| matches!(c, Some(1000) | Some(1001))),
            "no close frame on drop: {closes:?}"
        );
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dr_02_drop_during_backoff_makes_no_further_attempts() {
    guard(async {
        // Halfway through the pending delay, as in RT-DC-06.
        let (server, h) = crate::dc::in_backoff(crate::dc::PENDING / 2).await;
        // The server accepts again from the moment of the drop, so a
        // pending attempt that survived it would connect.
        let sync = h.into_sync().await;
        let attempts = server.connects();
        server.set_default(Behaviour::echo());
        drop(sync);
        tokio::time::sleep(MAX * 10).await;
        drain_real_io().await;
        assert_eq!(server.connects(), attempts, "attempt after drop");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dr_03_no_callbacks_after_drop() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().steps(vec![Step::TextEvery(
            "refresh-shopping-lists".into(),
            ms(2),
        )]));
        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);
        let mut sync =
            RealtimeSync::with_config(client_at(&server.http_url()), config(&server), move |_| {
                count2.fetch_add(1, Ordering::SeqCst);
            });
        sync.connect().await.unwrap();
        tokio::time::sleep(ms(100) + TICK).await;
        drop(sync);
        settle().await;
        let a = count.load(Ordering::SeqCst);
        assert_eq!(a, 51, "events before the drop");
        tokio::time::sleep(HB * 10).await;
        drain_real_io().await;
        assert_eq!(count.load(Ordering::SeqCst), a, "callback ran after drop");
        assert!(server.conn(0).client_closed_at.is_some());
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_dr_04_drop_inside_a_callback_spawned_task_does_not_deadlock() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let slot: Arc<Mutex<Option<RealtimeSync>>> = Arc::new(Mutex::new(None));
        let (done_tx, mut done_rx) = tokio::sync::mpsc::unbounded_channel();
        let slot2 = Arc::clone(&slot);
        let first = Arc::new(AtomicBool::new(true));
        let mut sync =
            RealtimeSync::with_config(client_at(&server.http_url()), config(&server), move |_| {
                if first.swap(false, Ordering::SeqCst) {
                    let slot = Arc::clone(&slot2);
                    let done = done_tx.clone();
                    tokio::spawn(async move {
                        let taken = slot.lock().unwrap().take();
                        drop(taken);
                        let _ = done.send(Instant::now());
                    });
                }
            });
        sync.connect().await.unwrap();
        *slot.lock().unwrap() = Some(sync);
        let t = Instant::now();
        server.send_text(0, "refresh-shopping-lists");
        let dropped_at = done_rx
            .recv()
            .await
            .expect("drop in spawned task did not return");
        assert_eq!(dropped_at, t, "drop in the spawned task waited");
        let c = server
            .wait_ended(0, CLOSE_TIMEOUT)
            .await
            .expect("server did not see the connection end within close_timeout");
        assert_eq!(c.ended_at, Some(t), "connection ended late");
    })
    .await
}
