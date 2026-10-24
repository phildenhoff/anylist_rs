//! 2.3 Liveness (LV)
//!
//! Liveness is checked on the heartbeat tick, so a dead connection is
//! detected between `liveness_timeout` and `liveness_timeout + one
//! heartbeat interval` after the last inbound frame. In virtual time the
//! tests assert exactly that window.

use crate::realtime::fake::{Behaviour, FakeServer, Step};
use crate::realtime::support::*;
use anylist_rs::{ConnectionState, SyncEvent};
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn rt_lv_01_a_silent_server_is_detected_within_a_bounded_time() {
    guard(async {
        // Connection 0: raw mode, reads and discards, never sends anything
        // (no echo, no Pong). Later attempts are rejected so the client stays
        // in Reconnecting.
        let server = FakeServer::start(Behaviour::reject(503));
        server.script([Behaviour::raw()]);
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();

        let seen = h
            .wait_state(ConnectionState::Reconnecting, LIVENESS * 2)
            .await
            .expect("silent server never detected as dead");
        let c = server.conn(0);
        let dropped = c.client_closed_at.expect("client kept the dead connection");
        assert_within(dropped - t0, LIVENESS, LIVENESS + HB, "detected after");
        assert_within(seen - dropped, ZERO, TICK, "Reconnecting after the drop");

        let c = server
            .wait_ended(0, TICK)
            .await
            .expect("server never saw connection 0 end");
        assert_eq!(c.ended_at, Some(dropped), "server saw the end late");
        assert_eq!(c.frames.len(), 0, "raw mode records nothing");
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_lv_02_any_inbound_frame_counts_as_liveness() {
    guard(async {
        // Raw mode: no echo and no Pong, only an event every
        // liveness_timeout - 1 s.
        let every = LIVENESS - secs(1);
        let server = FakeServer::start(
            Behaviour::raw().steps(vec![Step::TextEvery("refresh-list-settings".into(), every)]),
        );
        let h = Harness::for_server(&server);
        h.connect().await.unwrap();
        let t0 = Instant::now();
        let poller = h.poll_states(ms(10));
        tokio::time::sleep(HB * 20 + TICK).await;
        let samples = poller.samples();
        assert!(samples.len() >= 10_000, "{} samples", samples.len());
        assert_eq!(
            poller.dedup(),
            vec![ConnectionState::Connected],
            "left Connected"
        );
        let n = h.events.count(|e| *e == SyncEvent::ListSettingsChanged);
        let want = (HB * 20).as_millis() / every.as_millis() + 1;
        assert_eq!(
            n as u128,
            want,
            "ListSettingsChanged in {:?}",
            Instant::now() - t0
        );
        assert_eq!(server.connects(), 1);
    })
    .await
}

#[tokio::test(start_paused = true)]
async fn rt_lv_03_pong_and_ping_frames_count_as_liveness() {
    guard(async {
        // (a) Never sends text, answers the client's Pings (library default).
        let pong_server = FakeServer::start(Behaviour::silent());
        // (b) Raw mode, never answers, sends its own Ping every
        // liveness_timeout - 1 s.
        let ping_server =
            FakeServer::start(Behaviour::raw().steps(vec![Step::PingEvery(LIVENESS - secs(1))]));
        let a = Harness::for_server(&pong_server);
        let b = Harness::for_server(&ping_server);
        a.connect().await.unwrap();
        b.connect().await.unwrap();
        let pa = a.poll_states(ms(10));
        let pb = b.poll_states(ms(10));
        tokio::time::sleep(HB * 20 + TICK).await;
        for (name, p) in [("pong", &pa), ("ping", &pb)] {
            assert!(p.samples().len() >= 10_000);
            assert_eq!(
                p.dedup(),
                vec![ConnectionState::Connected],
                "{name} server: left Connected"
            );
        }
        assert_eq!(pong_server.connects(), 1);
        assert_eq!(ping_server.connects(), 1);
        assert_eq!(pong_server.conn(0).pings().len(), 20, "client Pings");
    })
    .await
}

/// Connection 0 echoes two heartbeats then goes quiet (half-open: TCP stays
/// open, nothing is written); connection 1 is normal, with a 50 ms
/// handshake.
async fn half_open_setup() -> (FakeServer, Harness, StatePoller) {
    let server = FakeServer::start(Behaviour::echo().delay_handshake(ms(50)));
    server.script([Behaviour::echo().echo_limit(2)]);
    let h = Harness::for_server(&server);
    h.connect().await.unwrap();
    let poller = h.poll_states(TICK);
    (server, h, poller)
}

#[tokio::test(start_paused = true)]
async fn rt_lv_04_half_open_connection_is_recovered() {
    guard(async {
        let (server, h, poller) = half_open_setup().await;
        let c1 = server
            .wait_upgraded(1, LIVENESS * 3)
            .await
            .expect("no second connection after the half-open one");
        // Let the poller sample the final state.
        tokio::time::sleep(ms(10)).await;

        let c0 = server.conn(0);
        let quiet_at = c0.quiet_at.expect("connection 0 never went quiet");
        let rec = poller
            .first_at_or_after(quiet_at, ConnectionState::Reconnecting)
            .expect("Reconnecting never observed");
        assert_within(
            rec - quiet_at,
            LIVENESS,
            LIVENESS + HB + TICK,
            "Reconnecting after the server went quiet",
        );
        let dropped = c0.client_closed_at.expect("dead connection never dropped");
        assert_within(rec - dropped, ZERO, TICK, "Reconnecting after the drop");
        assert!(
            c1.accepted_at - dropped <= INITIAL,
            "second attempt {:?} after the drop",
            c1.accepted_at - dropped
        );
        assert_eq!(c1.upgraded_at.unwrap() - c1.accepted_at, ms(50));
        let back = poller
            .first_at_or_after(c1.upgraded_at.unwrap(), ConnectionState::Connected)
            .expect("not Connected again");
        assert_within(back - c1.upgraded_at.unwrap(), ZERO, TICK, "Connected");
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

#[tokio::test(start_paused = true)]
async fn rt_lv_05_the_dead_socket_is_closed_before_the_new_one_opens() {
    guard(async {
        let (server, _h, _poller) = half_open_setup().await;
        let c1 = server
            .wait_upgraded(1, LIVENESS * 3)
            .await
            .expect("no second connection after the half-open one");
        let c0 = server.conn(0);
        let closed_seq = c0
            .client_closed_seq
            .expect("connection 0 still open when connection 1 was opened");
        assert!(
            closed_seq < c1.accepted_seq,
            "connection 1 opened before connection 0 was closed"
        );
        let ended = c0.ended_at.expect("server never saw connection 0 end");
        assert!(
            ended <= c1.accepted_at,
            "connection 0 ended after connection 1 opened"
        );
        assert_eq!(server.max_concurrent_open(), 1);
    })
    .await
}
