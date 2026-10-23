//! Shared helpers: test configuration, a harness around `RealtimeSync`, an
//! event recorder and polling helpers.

#![allow(dead_code)]

use crate::fake::FakeServer;
use anylist_rs::{
    AnyListClient, AnyListError, AuthEvent, ConnectionState, RealtimeConfig, RealtimeSync,
    SavedTokens, SyncEvent,
};
use futures_util::FutureExt;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub const HB: Duration = Duration::from_millis(50);
pub const LIVENESS: Duration = Duration::from_millis(150);
pub const INITIAL: Duration = Duration::from_millis(100);
pub const MAX: Duration = Duration::from_millis(400);
pub const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
pub const CLOSE_TIMEOUT: Duration = Duration::from_millis(500);
pub const SEND_TIMEOUT: Duration = Duration::from_millis(500);

pub const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Run a test body under a hard 10 s cap, so a hang is a failure.
pub async fn guard<F: Future<Output = ()>>(body: F) {
    tokio::time::timeout(Duration::from_secs(10), body)
        .await
        .expect("test body did not finish within 10 s");
}

/// Run independent sub-scenarios concurrently, each in its own task.
///
/// Building an `AnyListClient` (a `reqwest::Client`) is synchronous and
/// CPU-heavy in debug builds; if several runs shared one task with
/// `join!`, one run's client construction would stall another run's
/// in-flight handshake.
pub async fn concurrently<F>(runs: Vec<F>)
where
    F: Future<Output = ()> + Send + 'static,
{
    let handles: Vec<_> = runs.into_iter().map(tokio::spawn).collect();
    for h in handles {
        if let Err(e) = h.await {
            if e.is_panic() {
                std::panic::resume_unwind(e.into_panic());
            }
            panic!("sub-scenario failed: {e}");
        }
    }
}

/// The test configuration from section 3.4 of the behaviour spec.
pub fn config_for_url(ws_url: &str) -> RealtimeConfig {
    RealtimeConfig::default()
        .with_websocket_url(ws_url)
        .with_heartbeat_interval(HB)
        .with_liveness_timeout(LIVENESS)
        .with_reconnect_delays(INITIAL, MAX)
        .with_connect_timeout(CONNECT_TIMEOUT)
        .with_close_timeout(CLOSE_TIMEOUT)
        .with_send_timeout(SEND_TIMEOUT)
}

pub fn config(server: &FakeServer) -> RealtimeConfig {
    config_for_url(&server.ws_url())
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

    pub fn for_server(server: &FakeServer) -> Self {
        Self::new(client_at(&server.http_url()), config(server))
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

    /// Poll `state()` until it equals `want`; returns when it was first seen.
    pub async fn wait_state(&self, want: ConnectionState, within: Duration) -> Option<Instant> {
        let end = Instant::now() + within;
        loop {
            if self.state().await == want {
                return Some(Instant::now());
            }
            if Instant::now() >= end {
                return None;
            }
            tokio::time::sleep(ms(5)).await;
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

    /// Drop the `RealtimeSync` (returns how long `drop` took).
    pub async fn drop_sync(self) -> Duration {
        let Harness { sync, .. } = self;
        let mut sync = sync;
        loop {
            match Arc::try_unwrap(sync) {
                Ok(lock) => {
                    let inner = lock.into_inner();
                    let t = Instant::now();
                    drop(inner);
                    return t.elapsed();
                }
                Err(s) => {
                    sync = s;
                    tokio::time::sleep(ms(1)).await;
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

/// Poll `f` every 5 ms until it returns true or `within` passes.
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
        tokio::time::sleep(ms(5)).await;
    }
}

/// Assert `d` is within `pct` percent of `expected`.
#[track_caller]
pub fn assert_near(d: Duration, expected: Duration, pct: u32, what: &str) {
    let lo = expected * (100 - pct) / 100;
    let hi = expected * (100 + pct) / 100;
    assert!(
        d >= lo && d <= hi,
        "{what}: {d:?} not within {pct}% of {expected:?} ({lo:?}..={hi:?})"
    );
}

pub fn gaps(times: &[Instant]) -> Vec<Duration> {
    times.windows(2).map(|w| w[1] - w[0]).collect()
}
