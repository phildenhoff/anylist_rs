//! 2.4 Event delivery (EV)

use crate::realtime::fake::{Behaviour, FakeServer, Step, HEARTBEAT};
use crate::realtime::support::*;
use anylist_rs::{ConnectionState, RealtimeSync, SyncEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::Instant;

pub const KNOWN: [(&str, SyncEvent); 14] = [
    ("refresh-shopping-lists", SyncEvent::ShoppingListsChanged),
    (
        "refresh-categorized-items",
        SyncEvent::CategorizedItemsChanged,
    ),
    ("refresh-list-folders", SyncEvent::ListFoldersChanged),
    ("refresh-list-settings", SyncEvent::ListSettingsChanged),
    ("refresh-starter-lists", SyncEvent::StarterListsChanged),
    (
        "refresh-ordered-starter-list-ids",
        SyncEvent::StarterListOrderChanged,
    ),
    (
        "refresh-starter-list-settings",
        SyncEvent::StarterListSettingsChanged,
    ),
    (
        "refresh-mobile-app-settings",
        SyncEvent::MobileAppSettingsChanged,
    ),
    ("refresh-user-categories", SyncEvent::UserCategoriesChanged),
    ("refresh-user-recipe-data", SyncEvent::RecipeDataChanged),
    (
        "refresh-meal-plan-calendar",
        SyncEvent::MealPlanCalendarChanged,
    ),
    ("refresh-account-info", SyncEvent::AccountInfoChanged),
    (
        "refresh-subscription-info",
        SyncEvent::SubscriptionInfoChanged,
    ),
    ("did-delete-account", SyncEvent::AccountDeleted),
];

async fn connected() -> (FakeServer, Harness) {
    let server = FakeServer::start(Behaviour::echo());
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    (server, h)
}

#[tokio::test(start_paused = true)]
async fn rt_ev_01_each_known_message_maps_to_its_event() {
    guard(async {
        let (server, h) = connected().await;
        let mut sent = Vec::new();
        for (msg, _) in KNOWN.iter() {
            sent.push(Instant::now());
            server.send_text(0, msg);
            tokio::time::sleep(ms(5)).await;
        }
        let want: Vec<SyncEvent> = KNOWN.iter().map(|(_, e)| e.clone()).collect();
        assert_eq!(h.events.events(), want);
        // Each one delivered at the instant it was sent.
        let at: Vec<Instant> = h.events.all().iter().map(|r| r.at).collect();
        assert_eq!(at, sent);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_ev_02_heartbeats_are_never_delivered() {
    guard(async {
        // Echoed heartbeats plus unsolicited ones from the server.
        let server = FakeServer::start(Behaviour::echo().steps(vec![
            Step::Text(HEARTBEAT.into()),
            Step::Sleep(ms(20)),
            Step::Text(HEARTBEAT.into()),
            Step::Sleep(ms(20)),
            Step::Text(HEARTBEAT.into()),
        ]));
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        tokio::time::sleep(HB * 6 + TICK).await;
        // The client sent six heartbeats (all echoed back) and received
        // three unsolicited ones.
        assert_eq!(server.conn(0).heartbeats().len(), 6);
        assert_eq!(h.events.events(), Vec::<SyncEvent>::new());
        assert_eq!(h.state().await, ConnectionState::Connected);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_ev_03_unknown_text_is_ignored_and_the_session_survives() {
    guard(async {
        let (server, h) = connected().await;
        for msg in [
            "refresh-something-new",
            "{\"json\":true}",
            "",
            "refresh-shopping-lists",
        ] {
            server.send_text(0, msg);
        }
        settle().await;
        assert_eq!(h.events.events(), vec![SyncEvent::ShoppingListsChanged]);
        tokio::time::sleep(HB * 4).await;
        assert_eq!(h.events.events(), vec![SyncEvent::ShoppingListsChanged]);
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_eq!(server.connects(), 1);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_ev_04_binary_frames_are_ignored() {
    guard(async {
        let (server, h) = connected().await;
        server.send_binary(0, vec![7u8; 16]);
        server.send_text(0, "refresh-list-folders");
        settle().await;
        assert_eq!(h.events.events(), vec![SyncEvent::ListFoldersChanged]);
        tokio::time::sleep(HB * 4).await;
        assert_eq!(h.events.events(), vec![SyncEvent::ListFoldersChanged]);
        assert_eq!(h.state().await, ConnectionState::Connected);
        assert_eq!(server.connects(), 1);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_ev_05_ordering_is_preserved_under_a_burst() {
    guard(async {
        let (server, h) = connected().await;
        let mut want = Vec::new();
        let t = Instant::now();
        for i in 0..200 {
            let (msg, ev) = &KNOWN[i % KNOWN.len()];
            server.send_text(0, msg);
            want.push(ev.clone());
        }
        settle().await;
        assert_eq!(h.events.events(), want);
        assert!(h.events.all().iter().all(|r| r.at == t), "delivered late");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_ev_06_the_callback_runs_inside_the_tokio_runtime_and_may_spawn() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut sync = RealtimeSync::with_config(
            client_at(&server.http_url()),
            config(&server),
            move |event| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(event);
                });
            },
        );
        sync.connect().await.unwrap();
        let t = Instant::now();
        server.send_text(0, "refresh-shopping-lists");
        let got = rx.recv().await;
        assert_eq!(t.elapsed(), ZERO, "spawned task delivered late");
        assert_eq!(got, Some(SyncEvent::ShoppingListsChanged));
        sync.disconnect().await.unwrap();
    })
    .await
}

/// Real time: the callback really blocks its thread for 2 x heartbeat
/// interval, which a paused clock cannot represent (time does not pass
/// while a thread is blocked). The stall (1 s) is well below the liveness
/// timeout (5 s), and nothing here depends on scheduling latency.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_ev_07_a_slow_callback_delays_but_does_not_drop_later_events() {
    guard_real(async {
        let server = FakeServer::start(Behaviour::echo());
        let got = Arc::new(Mutex::new(Vec::new()));
        let got2 = Arc::clone(&got);
        let first = Arc::new(AtomicBool::new(true));
        let stall = real::HB * 2;
        let mut sync = RealtimeSync::with_config(
            client_at(&server.http_url()),
            real_config(&server),
            move |event| {
                if first.swap(false, Ordering::SeqCst) {
                    std::thread::sleep(stall);
                }
                got2.lock().unwrap().push((Instant::now(), event));
            },
        );
        sync.connect().await.unwrap();
        let t = Instant::now();
        for msg in [
            "refresh-shopping-lists",
            "refresh-list-folders",
            "refresh-account-info",
        ] {
            server.send_text(0, msg);
            tokio::time::sleep(ms(10)).await;
        }
        let mut states = Vec::new();
        while t.elapsed() < real::LIVENESS && got.lock().unwrap().len() < 3 {
            states.push(sync.state().await);
            tokio::time::sleep(ms(10)).await;
        }
        let got = got.lock().unwrap().clone();
        assert_eq!(
            got.iter().map(|(_, e)| e.clone()).collect::<Vec<_>>(),
            vec![
                SyncEvent::ShoppingListsChanged,
                SyncEvent::ListFoldersChanged,
                SyncEvent::AccountInfoChanged
            ]
        );
        // The stall really happened, and delayed the later events.
        assert!(got[0].0 - t >= stall, "first callback returned early");
        assert!(got[1].0 >= got[0].0 && got[2].0 >= got[1].0);
        states.push(sync.state().await);
        assert!(
            states.iter().all(|s| *s == ConnectionState::Connected),
            "{states:?}"
        );
        assert_eq!(server.connects(), 1);
        sync.disconnect().await.unwrap();
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_ev_08_no_events_from_a_dead_session_leak_after_the_state_changes() {
    guard(async {
        let server = FakeServer::start(Behaviour::echo());
        server.script([Behaviour::echo().steps(vec![
            Step::Sleep(ms(50)),
            Step::Text("refresh-shopping-lists".into()),
            Step::Close(1000),
        ])]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        assert!(server.wait_upgraded(1, secs(1)).await.is_some());
        tokio::time::sleep(HB * 2).await;
        let recs = h.events.all();
        assert_eq!(recs.len(), 2, "{:?}", h.events.events());
        assert_eq!(recs[0].event, SyncEvent::ShoppingListsChanged);
        assert!(is_reconnected(&recs[1].event), "{:?}", recs[1].event);
        for r in &recs {
            assert_eq!(
                r.state,
                Some(ConnectionState::Connected),
                "{:?} delivered while state was {:?}",
                r.event,
                r.state
            );
        }
    })
    .await
}
