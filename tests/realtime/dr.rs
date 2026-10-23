//! 2.8 Drop (DR)

use crate::fake::{Behaviour, FakeServer, Step};
use crate::support::*;
use anylist_rs::RealtimeSync;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dr_01_drop_while_connected_ends_the_connection() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo()).await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t = Instant::now();
        let took = h.drop_sync().await;
        assert!(took <= ms(10), "drop took {took:?}");
        let c = server
            .wait_ended(0, CLOSE_TIMEOUT + ms(200))
            .await
            .expect("server did not see the connection end within 700 ms");
        assert!(c.ended_at.unwrap() - t <= CLOSE_TIMEOUT + ms(200));
        // SHOULD: a close frame rather than a bare EOF.
        let closes = c.close_frames();
        assert!(
            closes.iter().any(|c| matches!(c, Some(1000) | Some(1001))),
            "no close frame on drop: {closes:?}"
        );
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dr_02_drop_during_backoff_makes_no_further_attempts() {
    guard(async {
        // Max delay shortened to 200 ms so 10 x reconnect_max_delay fits the
        // per-test budget.
        let max = ms(200);
        let server = FakeServer::start(Behaviour::reject(503)).await;
        server.script([Behaviour::echo().steps(vec![Step::Sleep(ms(200)), Step::DropTcp])]);
        let h = Harness::new(
            client_at(&server.http_url()),
            config(&server).with_reconnect_delays(INITIAL, max),
        );
        h.connect().await.unwrap();
        server
            .wait_attempt(3, ms(2000))
            .await
            .expect("three failed attempts");
        tokio::time::sleep(ms(10)).await;
        let _ = h.drop_sync().await;
        let attempts = server.attempts();
        server.set_default(Behaviour::echo());
        tokio::time::sleep(max * 10).await;
        assert_eq!(server.attempts(), attempts, "attempt after drop");
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dr_03_no_callbacks_after_drop() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo().steps(vec![Step::TextEvery(
            "refresh-shopping-lists".into(),
            ms(2),
        )]))
        .await;
        let count = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::clone(&count);
        let mut sync =
            RealtimeSync::with_config(client_at(&server.http_url()), config(&server), move |_| {
                count2.fetch_add(1, Ordering::SeqCst);
            });
        sync.connect().await.unwrap();
        tokio::time::sleep(ms(100)).await;
        drop(sync);
        tokio::time::sleep(ms(50)).await;
        let a = count.load(Ordering::SeqCst);
        assert!(a > 0);
        tokio::time::sleep(ms(500)).await;
        assert_eq!(count.load(Ordering::SeqCst), a, "callback ran after drop");
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_dr_04_drop_inside_a_callback_spawned_task_does_not_deadlock() {
    guard(async {
        let t = Instant::now();
        let server = FakeServer::start(Behaviour::echo()).await;
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
                        let _ = done.send(());
                    });
                }
            });
        sync.connect().await.unwrap();
        *slot.lock().unwrap() = Some(sync);
        server.send_text(0, "refresh-shopping-lists");
        tokio::time::timeout(ms(1000), done_rx.recv())
            .await
            .expect("drop in spawned task did not return");
        assert!(
            server.wait_ended(0, ms(700)).await.is_some(),
            "server did not see the connection end within 700 ms"
        );
        assert!(t.elapsed() <= ms(2000), "took {:?}", t.elapsed());
    })
    .await
}
