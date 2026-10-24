//! Shared helpers: test configurations, a harness around `RealtimeSync`, an
//! event recorder, state pollers and the test guard.
//!
//! All times are `tokio::time::Instant`, so they follow the paused clock in
//! virtual-time tests and the real clock otherwise.

#![allow(dead_code)]

use crate::fake::FakeServer;
use anylist_rs::{
    AnyListClient, AnyListError, AuthEvent, ConnectionState, RealtimeConfig, RealtimeSync,
    SavedTokens, SyncEvent,
};
use futures_util::FutureExt;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

// Production defaults (`RealtimeConfig::default()`), used by every
// virtual-time test. `config()` checks they still match.
pub const HB: Duration = Duration::from_secs(5);
pub const LIVENESS: Duration = Duration::from_secs(15);
pub const INITIAL: Duration = Duration::from_secs(2);
pub const MAX: Duration = Duration::from_secs(120);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);
pub const SEND_TIMEOUT: Duration = Duration::from_secs(5);

/// Resolution of the state pollers in virtual-time tests.
pub const TICK: Duration = Duration::from_millis(1);

// Real-time tests (the ones that need the HTTP refresh endpoint, a real
// blocking callback or real TCP). Sized so that no assertion depends on
// scheduling latency.
pub mod real {
    use std::time::Duration;
    pub const HB: Duration = Duration::from_millis(500);
    pub const LIVENESS: Duration = Duration::from_secs(5);
    pub const INITIAL: Duration = Duration::from_millis(500);
    pub const MAX: Duration = Duration::from_secs(4);
    pub const TIMEOUT: Duration = Duration::from_secs(3);
}

pub const ZERO: Duration = Duration::ZERO;

pub const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}
pub const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

/// Kill the whole test binary if a test is still running after this much
/// real time. A thread that blocks (rather than awaits) would otherwise
/// hang the run: a tokio timeout cannot fire on a blocked thread.
const WATCHDOG: Duration = Duration::from_secs(60);

fn watchdog() -> std::sync::mpsc::Sender<()> {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let name = std::thread::current()
        .name()
        .unwrap_or("<unnamed>")
        .to_string();
    std::thread::spawn(move || {
        if let Err(std::sync::mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(WATCHDOG) {
            eprintln!("{name}: still running after {WATCHDOG:?} of real time, aborting");
            std::process::abort();
        }
    });
    tx
}

/// Run a virtual-time test body. A logical hang shows up at once (the
/// paused clock jumps to the cap) and fails the test; a blocked thread is
/// caught by the real-time watchdog.
pub async fn guard<F: Future<Output = ()>>(body: F) {
    let _dog = watchdog();
    tokio::time::timeout(Duration::from_secs(4 * 3600), body)
        .await
        .expect("test body did not finish within 4 h of virtual time");
}

/// Run a real-time test body under a 30 s cap.
pub async fn guard_real<F: Future<Output = ()>>(body: F) {
    let _dog = watchdog();
    tokio::time::timeout(Duration::from_secs(30), body)
        .await
        .expect("test body did not finish within 30 s");
}

/// Let everything runnable at the current instant run to completion. In a
/// virtual-time test the paused clock only moves once every task is idle,
/// so after this returns, everything triggered "now" has happened (the
/// clock moves on by 1 ms).
pub async fn settle() {
    tokio::time::sleep(TICK).await;
}

/// Sleep until `t`.
pub async fn until(t: Instant) {
    tokio::time::sleep_until(t).await;
}

/// The production configuration, connected to the in-memory fake.
pub fn config(server: &FakeServer) -> RealtimeConfig {
    let c = RealtimeConfig::default();
    assert_eq!(c.heartbeat_interval, HB);
    assert_eq!(c.liveness_timeout, LIVENESS);
    assert_eq!(c.reconnect_initial_delay, INITIAL);
    assert_eq!(c.reconnect_max_delay, MAX);
    assert_eq!(c.connect_timeout, CONNECT_TIMEOUT);
    assert_eq!(c.close_timeout, CLOSE_TIMEOUT);
    assert_eq!(c.send_timeout, SEND_TIMEOUT);
    c.with_transport(server.transport())
}

/// The real-time configuration (see [`real`]), connected to the in-memory
/// fake.
pub fn real_config(server: &FakeServer) -> RealtimeConfig {
    RealtimeConfig::default()
        .with_heartbeat_interval(real::HB)
        .with_liveness_timeout(real::LIVENESS)
        .with_reconnect_delays(real::INITIAL, real::MAX)
        .with_connect_timeout(real::TIMEOUT)
        .with_close_timeout(real::TIMEOUT)
        .with_send_timeout(real::TIMEOUT)
        .with_transport(server.transport())
}

/// A client with tokens ("access-1", "refresh-1") whose HTTP base URL points
/// at `http_base`.
pub fn client_at(http_base: &str) -> Arc<AnyListClient> {
    Arc::new(new_client(http_base))
}

pub fn new_client(http_base: &str) -> AnyListClient {
    let tokens = SavedTokens::new("access-1", "refresh-1", "user-1", false);
    AnyListClient::from_tokens(tokens)
        .unwrap()
        .with_base_url(http_base)
}

/// A client that records its auth events.
pub fn client_with_auth_log(http_base: &str) -> (Arc<AnyListClient>, Arc<Mutex<Vec<AuthEvent>>>) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let log2 = Arc::clone(&log);
    let client = new_client(http_base).on_auth_event(move |e| log2.lock().unwrap().push(e));
    (Arc::new(client), log)
}

/// A bound-then-released port with nothing listening on it.
pub async fn free_port() -> u16 {
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    l.local_addr().unwrap().port()
}

/// Give real I/O that is still in flight a moment of real time to land.
///
/// Only the refresh endpoint is real I/O (reqwest over TCP). A client that
/// wrongly starts a refresh in a virtual-time test can be overtaken by the
/// paused clock, which keeps jumping ahead while the kernel delivers bytes.
/// Before asserting that something did *not* happen, the clock is resumed
/// for 50 ms of real time so that such a refresh, and whatever the client
/// does after it, shows up. This can only make a check stricter: a correct
/// client has nothing in flight.
///
/// Afterwards the clock is moved on to the next whole millisecond, so that
/// instants stay on the timer wheel's 1 ms grid and later timings remain
/// exact.
pub async fn drain_real_io() {
    let before = Instant::now();
    tokio::time::resume();
    tokio::time::sleep(ms(50)).await;
    tokio::time::pause();
    let gone = Instant::now() - before;
    let whole = Duration::from_millis(gone.as_millis() as u64 + 1);
    tokio::time::advance(whole - gone).await;
}

/// Assert that the client never tried to refresh its tokens (the refresh
/// endpoint counts every TCP connection, even one that never sends a
/// request).
pub async fn assert_no_refresh(server: &FakeServer) {
    drain_real_io().await;
    assert_eq!(
        server.refresh_connections(),
        0,
        "the client tried to refresh its tokens"
    );
}

pub fn is_reconnected(e: &SyncEvent) -> bool {
    matches!(e, SyncEvent::Reconnected)
}

#[derive(Clone, Debug)]
pub struct Rec {
    pub at: Instant,
    pub event: SyncEvent,
    /// `state()` read from inside the callback (None if it could not be read
    /// without blocking).
    pub state: Option<ConnectionState>,
}

#[derive(Clone, Default)]
pub struct Events(Arc<Mutex<Vec<Rec>>>);

impl Events {
    pub fn all(&self) -> Vec<Rec> {
        self.0.lock().unwrap().clone()
    }
    pub fn events(&self) -> Vec<SyncEvent> {
        self.all().into_iter().map(|r| r.event).collect()
    }
    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }
    pub fn count(&self, pred: impl Fn(&SyncEvent) -> bool) -> usize {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter(|r| pred(&r.event))
            .count()
    }
    pub fn reconnected(&self) -> usize {
        self.count(is_reconnected)
    }
}

pub type Shared = Arc<RwLock<RealtimeSync>>;

/// A `RealtimeSync` behind a `RwLock`, with a callback that records every
/// event together with the state read from inside the callback.
pub struct Harness {
    pub client: Arc<AnyListClient>,
    pub sync: Shared,
    pub events: Events,
}

impl Harness {
    pub fn new(client: Arc<AnyListClient>, config: RealtimeConfig) -> Self {
        let events = Events::default();
        let cell: Arc<OnceLock<Weak<RwLock<RealtimeSync>>>> = Arc::new(OnceLock::new());
        let ev = events.clone();
        let cell2 = Arc::clone(&cell);
        let sync = RealtimeSync::with_config(Arc::clone(&client), config, move |event| {
            let state = cell2
                .get()
                .and_then(|w| w.upgrade())
                .and_then(|s| s.try_read().ok().and_then(|g| g.state().now_or_never()));
            ev.0.lock().unwrap().push(Rec {
                at: Instant::now(),
                event,
                state,
            });
        });
        let sync = Arc::new(RwLock::new(sync));
        let _ = cell.set(Arc::downgrade(&sync));
        Self {
            client,
            sync,
            events,
        }
    }

    /// Production config over the in-memory fake (virtual-time tests).
    pub fn for_server(server: &FakeServer) -> Self {
        Self::new(client_at(&server.http_url()), config(server))
    }

    /// Real-time config over the in-memory fake.
    pub fn real(server: &FakeServer) -> Self {
        Self::new(client_at(&server.http_url()), real_config(server))
    }

    pub async fn connect(&self) -> Result<(), AnyListError> {
        self.sync.write().await.connect().await
    }
    pub async fn disconnect(&self) -> Result<(), AnyListError> {
        self.sync.write().await.disconnect().await
    }
    pub async fn reconnect(&self) -> Result<(), AnyListError> {
        self.sync.write().await.reconnect().await
    }
    pub async fn state(&self) -> ConnectionState {
        self.sync.read().await.state().await
    }
    pub async fn is_connected(&self) -> bool {
        self.sync.read().await.is_connected().await
    }

    /// Poll `state()` every [`TICK`] until it equals `want`; returns when it
    /// was first seen.
    pub async fn wait_state(&self, want: ConnectionState, within: Duration) -> Option<Instant> {
        let end = Instant::now() + within;
        loop {
            if self.state().await == want {
                return Some(Instant::now());
            }
            if Instant::now() >= end {
                return None;
            }
            tokio::time::sleep(TICK).await;
        }
    }

    /// Start a poller recording `state()` every `every`.
    pub fn poll_states(&self, every: Duration) -> StatePoller {
        let sync = Arc::clone(&self.sync);
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = Arc::clone(&log);
        let task = tokio::spawn(async move {
            loop {
                let s = sync.read().await.state().await;
                log2.lock().unwrap().push((Instant::now(), s));
                tokio::time::sleep(every).await;
            }
        });
        StatePoller { log, task }
    }

    /// Take the `RealtimeSync` out of the harness (waits until no poller or
    /// callback holds a reference).
    pub async fn into_sync(self) -> RealtimeSync {
        let Harness { sync, .. } = self;
        let mut sync = sync;
        loop {
            match Arc::try_unwrap(sync) {
                Ok(lock) => return lock.into_inner(),
                Err(s) => {
                    sync = s;
                    tokio::time::sleep(TICK).await;
                }
            }
        }
    }
}

pub struct StatePoller {
    log: Arc<Mutex<Vec<(Instant, ConnectionState)>>>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for StatePoller {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl StatePoller {
    pub fn samples(&self) -> Vec<(Instant, ConnectionState)> {
        self.log.lock().unwrap().clone()
    }
    /// The sequence with consecutive duplicates removed.
    pub fn dedup(&self) -> Vec<ConnectionState> {
        let mut v: Vec<ConnectionState> = self.samples().into_iter().map(|(_, s)| s).collect();
        v.dedup();
        v
    }
    pub fn first_at_or_after(&self, t: Instant, s: ConnectionState) -> Option<Instant> {
        self.samples()
            .into_iter()
            .find(|(at, st)| *at >= t && *st == s)
            .map(|(at, _)| at)
    }
    pub fn ever(&self, s: ConnectionState) -> bool {
        self.samples().iter().any(|(_, st)| *st == s)
    }
    pub fn between(&self, from: Instant, to: Instant) -> Vec<ConnectionState> {
        self.samples()
            .into_iter()
            .filter(|(at, _)| *at >= from && *at <= to)
            .map(|(_, s)| s)
            .collect()
    }
}

/// Poll `f` every [`TICK`] until it returns true or `within` passes.
pub async fn eventually<F, Fut>(within: Duration, mut f: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let end = Instant::now() + within;
    loop {
        if f().await {
            return true;
        }
        if Instant::now() >= end {
            return false;
        }
        tokio::time::sleep(TICK).await;
    }
}

/// Assert `t` lies in `lo..=hi`.
#[track_caller]
pub fn assert_within(d: Duration, lo: Duration, hi: Duration, what: &str) {
    assert!(d >= lo && d <= hi, "{what}: {d:?} not in {lo:?}..={hi:?}");
}

pub fn gaps(times: &[Instant]) -> Vec<Duration> {
    times.windows(2).map(|w| w[1] - w[0]).collect()
}
