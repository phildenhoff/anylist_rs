//! 2.3 Liveness (LV)

use crate::fake::{Behaviour, FakeServer, Step};
use crate::support::*;
use anylist_rs::{ConnectionState, SyncEvent};
use std::time::Instant;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_lv_01_a_silent_server_is_detected_within_a_bounded_time() {
    guard(async {
        // Connection 0: raw mode, reads and discards, never sends anything
        // (no echo, no Pong). Later attempts are rejected so the client stays
        // in Reconnecting long enough to observe.
        let server = FakeServer::start(Behaviour::reject(503)).await;
        server.script([Behaviour::raw()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();

        let seen = h
            .wait_state(ConnectionState::Reconnecting, ms(600))
            .await
            .expect("silent server never detected as dead");
        let after = seen - t0;
        let upper = LIVENESS + HB * 2 + ms(50);
        assert!(after <= upper, "detected after {after:?}, bound {upper:?}");
        assert!(
            after >= LIVENESS * 8 / 10,
            "detected after {after:?}, too early"
        );

        let c = server
            .wait_ended(0, ms(300))
            .await
            .expect("server never saw connection 0 end");
        let ended = c.ended_at.unwrap();
        assert!(
            ended <= seen + ms(100),
            "connection ended {:?} after Reconnecting was observed",
            ended - seen
        );
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_lv_02_any_inbound_frame_counts_as_liveness() {
    guard(async {
        // Raw mode: no echo and no Pong, only an event every 100 ms.
        let server = FakeServer::start(Behaviour::raw().steps(vec![Step::TextEvery(
            "refresh-list-settings".into(),
            ms(100),
        )]))
        .await;
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let poller = h.poll_states(ms(10));
        tokio::time::sleep(ms(1000)).await;
        let samples = poller.samples();
        assert!(samples.len() > 50);
        for (_, s) in &samples {
            assert_eq!(*s, ConnectionState::Connected, "left Connected");
        }
        let n = h.events.count(|e| *e == SyncEvent::ListSettingsChanged);
        assert!(n >= 8, "only {n} ListSettingsChanged");
        assert_eq!(server.attempts(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_lv_03_pong_and_ping_frames_count_as_liveness() {
    guard(async {
        // (a) Never sends text, answers the client's Pings (library default).
        let pong_server = FakeServer::start(Behaviour::silent()).await;
        // (b) Raw mode, never answers, sends its own Ping every 100 ms.
        let ping_server =
            FakeServer::start(Behaviour::raw().steps(vec![Step::PingEvery(ms(100))])).await;
        let a = Harness::for_server(&pong_server);
        let b = Harness::for_server(&ping_server);
        a.connect().await.unwrap();
        b.connect().await.unwrap();
        let pa = a.poll_states(ms(10));
        let pb = b.poll_states(ms(10));
        tokio::time::sleep(ms(1000)).await;
        for (name, p) in [("pong", &pa), ("ping", &pb)] {
            let samples = p.samples();
            assert!(samples.len() > 50);
            assert!(
                samples
                    .iter()
                    .all(|(_, s)| *s == ConnectionState::Connected),
                "{name} server: left Connected: {:?}",
                p.dedup()
            );
        }
        assert_eq!(pong_server.attempts(), 1);
        assert_eq!(ping_server.attempts(), 1);
    })
    .await
}

/// Connection 0 echoes two heartbeats then goes quiet (half-open);
/// connection 1 is normal (with a short handshake delay so the poller can see
/// `Reconnecting` and the accept is ordered after connection 0's end).
async fn half_open_setup() -> (FakeServer, Harness, StatePoller) {
    let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(50))).await;
    server.script([Behaviour::echo().echo_limit(2)]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    let poller = h.poll_states(ms(5));
    (server, h, poller)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_lv_04_half_open_connection_is_recovered() {
    guard(async {
        let (server, h, poller) = half_open_setup().await;
        let c1 = server
            .wait_upgraded(1, ms(1500))
            .await
            .expect("no second connection after the half-open one");
        assert!(h
            .wait_state(ConnectionState::Connected, ms(300))
            .await
            .is_some());
        assert!(eventually(ms(300), || async { h.events.reconnected() >= 1 }).await);
        tokio::time::sleep(ms(100)).await;

        let quiet_at = server
            .conn(0)
            .quiet_at
            .expect("connection 0 never went quiet");
        let rec = poller
            .first_at_or_after(quiet_at, ConnectionState::Reconnecting)
            .expect("Reconnecting never observed");
        assert!(
            rec - quiet_at <= ms(300),
            "Reconnecting observed {:?} after the server went quiet",
            rec - quiet_at
        );
        let established = c1.upgraded_at.unwrap();
        assert!(
            established <= rec + INITIAL + ms(300),
            "second connection established {:?} after Reconnecting",
            established - rec
        );
        assert_eq!(
            poller.dedup(),
            vec![
                ConnectionState::Connected,
                ConnectionState::Reconnecting,
                ConnectionState::Connected
            ]
        );
        assert_eq!(h.events.reconnected(), 1);
        assert_eq!(server.max_concurrent_open(), 1);
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rt_lv_05_the_dead_socket_is_closed_before_the_new_one_opens() {
    guard(async {
        let (server, _h, _poller) = half_open_setup().await;
        let c1 = server
            .wait_upgraded(1, ms(1500))
            .await
            .expect("no second connection after the half-open one");
        let c0 = server.conn(0);
        let ended = c0
            .ended_at
            .expect("connection 0 still open when connection 1 was accepted");
        assert!(
            ended < c1.upgraded_at.unwrap(),
            "connection 0 ended after connection 1 was accepted"
        );
    })
    .await
}
