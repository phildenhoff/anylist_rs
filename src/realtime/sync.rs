//! The connection actor behind [`RealtimeSync`].
//!
//! One background task owns the WebSocket outright. Its loop is: run a
//! session (a `select!` over inbound frames, the heartbeat tick and the
//! command channel), then, unless the consumer closed it, back off and
//! reconnect. The backoff sleep, token refresh and connection attempts all
//! run inside a `select!` with the command channel, so `disconnect()` and
//! `Drop` are always handled promptly. State is published through a
//! `tokio::sync::watch` channel.

use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::realtime::config::RealtimeConfig;
use crate::realtime::events::SyncEvent;
use crate::realtime::reconnect::ReconnectionStrategy;
use crate::realtime::SyncCallback;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Instant, MissedTickBehavior};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{client_async, connect_async};

/// Connection state for real-time sync
///
/// Transitions: `Disconnected → Connecting → Connected` on the first
/// `connect()` (or back to `Disconnected` if it fails);
/// `Connected ⇄ Reconnecting` while the connection is interrupted and
/// restored; any state `→ Closed` on `disconnect()` or drop. Once
/// connected, the state never returns to `Disconnected` or `Connecting`,
/// and `Closed` is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionState {
    /// Never connected, or the first `connect()` failed
    Disconnected,
    /// The first `connect()` is in progress
    Connecting,
    /// Connected and active
    Connected,
    /// The connection was interrupted; backing off, refreshing tokens or
    /// attempting to reconnect
    Reconnecting,
    /// Permanently closed by `disconnect()` or drop
    Closed,
}

/// An open WebSocket, whatever byte stream it runs over (TCP/TLS by
/// default, or the stream from the `RealtimeConfig::with_transport` test
/// seam).
trait WsConn:
    Stream<Item = std::result::Result<Message, tungstenite::Error>>
    + Sink<Message, Error = tungstenite::Error>
    + Send
    + Unpin
{
}

impl<T> WsConn for T where
    T: Stream<Item = std::result::Result<Message, tungstenite::Error>>
        + Sink<Message, Error = tungstenite::Error>
        + Send
        + Unpin
{
}

type WsStream = Box<dyn WsConn>;
type StateTx = Arc<watch::Sender<ConnectionState>>;

const HEARTBEAT: &str = "--heartbeat--";

enum Command {
    /// Close the connection for good.
    Close,
    /// End the current session (or skip the backoff delay) and reconnect now.
    Reconnect,
}

struct ActorHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
    join: JoinHandle<()>,
}

/// Real-time sync manager
///
/// Callbacks are invoked synchronously on the connection's background
/// task. A callback that blocks also delays heartbeats and liveness checks,
/// so do real work elsewhere (for example in a `tokio::spawn`ed task).
pub struct RealtimeSync {
    client: Arc<AnyListClient>,
    config: RealtimeConfig,
    callback: SyncCallback,
    state: StateTx,
    actor: Option<ActorHandle>,
}

/// Move to `new` unless the state is already `Closed` (terminal).
fn set_state(tx: &watch::Sender<ConnectionState>, new: ConnectionState) {
    tx.send_if_modified(|s| {
        if *s == ConnectionState::Closed || *s == new {
            false
        } else {
            *s = new;
            true
        }
    });
}

/// Resets `Connecting` to `Disconnected` if `connect()` is cancelled or fails.
struct ConnectingGuard(Option<StateTx>);

impl Drop for ConnectingGuard {
    fn drop(&mut self) {
        if let Some(tx) = self.0.take() {
            tx.send_if_modified(|s| {
                if *s == ConnectionState::Connecting {
                    *s = ConnectionState::Disconnected;
                    true
                } else {
                    false
                }
            });
        }
    }
}

impl RealtimeSync {
    /// Create a new real-time sync instance, without connecting
    ///
    /// Call `connect()` to establish the WebSocket connection.
    pub fn new<F>(client: Arc<AnyListClient>, callback: F) -> Self
    where
        F: Fn(SyncEvent) + Send + Sync + 'static,
    {
        Self::with_config(client, RealtimeConfig::default(), callback)
    }

    /// Create a new real-time sync instance with custom settings, without
    /// connecting.
    ///
    /// An invalid configuration (zero heartbeat interval or liveness
    /// timeout, or an initial reconnect delay above the maximum) makes
    /// `connect()` return `Err(AnyListError::Other)`.
    pub fn with_config<F>(client: Arc<AnyListClient>, config: RealtimeConfig, callback: F) -> Self
    where
        F: Fn(SyncEvent) + Send + Sync + 'static,
    {
        let (state, _) = watch::channel(ConnectionState::Disconnected);
        Self {
            client,
            config,
            callback: Arc::new(callback),
            state: Arc::new(state),
            actor: None,
        }
    }

    /// Connect to the WebSocket server.
    ///
    /// Waits for the first handshake and returns its result: on `Err` the
    /// state is `Disconnected` and nothing keeps running. If the handshake is
    /// rejected with HTTP 401, the tokens are refreshed once and the
    /// handshake retried (a failed refresh returns
    /// `Err(AuthenticationFailed)`). Once connected, interruptions are
    /// handled in the background.
    ///
    /// Idempotent (returns `Ok(())`) while connected or reconnecting;
    /// returns `Err(NetworkError)` after `disconnect()`. Dropping the
    /// returned future abandons the attempt and leaves `Disconnected`.
    pub async fn connect(&mut self) -> Result<()> {
        match *self.state.borrow() {
            ConnectionState::Closed => {
                return Err(AnyListError::NetworkError(
                    "Cannot connect after permanent closure".to_string(),
                ))
            }
            ConnectionState::Connected | ConnectionState::Reconnecting => return Ok(()),
            ConnectionState::Disconnected | ConnectionState::Connecting => {}
        }
        self.config.validate()?;

        set_state(&self.state, ConnectionState::Connecting);
        let mut guard = ConnectingGuard(Some(Arc::clone(&self.state)));

        let ws = first_connect(&self.client, &self.config).await?;

        guard.0 = None;
        set_state(&self.state, ConnectionState::Connected);

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let ctx = ActorContext {
            client: Arc::clone(&self.client),
            config: self.config.clone(),
            callback: Arc::clone(&self.callback),
            state: Arc::clone(&self.state),
        };
        let join = tokio::spawn(ctx.run(ws, cmd_rx));
        self.actor = Some(ActorHandle { cmd_tx, join });
        Ok(())
    }

    /// The current connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.borrow()
    }

    /// Whether the state is `Connected`.
    pub async fn is_connected(&self) -> bool {
        *self.state.borrow() == ConnectionState::Connected
    }

    /// Disconnect and stop the background task.
    ///
    /// Sends a close frame (code 1000) if connected, aborts any backoff or
    /// connection attempt in flight, and returns within about
    /// `close_timeout`. The state becomes `Closed` for good. Never fails in
    /// practice; idempotent. No callback runs after this returns.
    pub async fn disconnect(&mut self) -> Result<()> {
        set_state(&self.state, ConnectionState::Closed);
        if let Some(actor) = self.actor.take() {
            let _ = actor.cmd_tx.send(Command::Close);
            let mut join = actor.join;
            if timeout(self.config.close_timeout, &mut join).await.is_err() {
                join.abort();
                let _ = join.await;
            }
        }
        Ok(())
    }

    /// Manually trigger reconnection.
    ///
    /// While connected, ends the current session (close frame) and
    /// reconnects at once. While reconnecting, skips the remaining backoff
    /// delay. Before the first connection, behaves as `connect()`. After
    /// `disconnect()`, returns `Err(NetworkError)`.
    pub async fn reconnect(&mut self) -> Result<()> {
        let state = *self.state.borrow();
        match state {
            ConnectionState::Closed => Err(AnyListError::NetworkError(
                "Cannot reconnect after permanent closure".to_string(),
            )),
            ConnectionState::Disconnected | ConnectionState::Connecting => self.connect().await,
            ConnectionState::Connected | ConnectionState::Reconnecting => {
                if let Some(actor) = &self.actor {
                    let _ = actor.cmd_tx.send(Command::Reconnect);
                }
                Ok(())
            }
        }
    }
}

impl Drop for RealtimeSync {
    /// Requests shutdown without waiting: the background task sends a
    /// best-effort close frame and exits. No further reconnect attempts are
    /// made.
    fn drop(&mut self) {
        set_state(&self.state, ConnectionState::Closed);
        if let Some(actor) = self.actor.take() {
            let _ = actor.cmd_tx.send(Command::Close);
        }
    }
}

/// Why a connection attempt failed.
enum ConnectError {
    /// The upgrade was rejected with HTTP 401.
    Unauthorized,
    Failed(String),
}

fn listener_url(client: &AnyListClient, config: &RealtimeConfig) -> Result<String> {
    let tokens = client.export_tokens()?;
    Ok(format!(
        "{}/data/add-user-listener?client_id={}&access_token={}",
        config.resolve_websocket_base(client.base_url()),
        urlencoding::encode(client.client_identifier()),
        urlencoding::encode(tokens.access_token())
    ))
}

/// Open the byte stream and run the WebSocket handshake over it: TCP/TLS
/// through `connect_async`, or the injected transport and `client_async`.
async fn handshake(
    url: &str,
    config: &RealtimeConfig,
) -> std::result::Result<WsStream, tungstenite::Error> {
    match &config.transport {
        None => {
            let (ws, _) = connect_async(url).await?;
            Ok(Box::new(ws))
        }
        Some(transport) => {
            let stream = (transport.0)(url).await?;
            let (ws, _) = client_async(url, stream).await?;
            Ok(Box::new(ws))
        }
    }
}

/// One WebSocket handshake, bounded by `connect_timeout`.
async fn open(
    client: &AnyListClient,
    config: &RealtimeConfig,
) -> std::result::Result<WsStream, ConnectError> {
    let url = listener_url(client, config).map_err(|e| ConnectError::Failed(e.to_string()))?;
    match timeout(config.connect_timeout, handshake(&url, config)).await {
        Err(_) => Err(ConnectError::Failed(format!(
            "WebSocket handshake timed out after {:?}",
            config.connect_timeout
        ))),
        Ok(Err(tungstenite::Error::Http(response))) if response.status() == 401 => {
            Err(ConnectError::Unauthorized)
        }
        Ok(Err(e)) => Err(ConnectError::Failed(format!(
            "WebSocket connection failed: {}",
            e
        ))),
        Ok(Ok(ws)) => Ok(ws),
    }
}

/// Refresh tokens, bounded by `connect_timeout`.
async fn refresh(client: &AnyListClient, config: &RealtimeConfig) -> Result<()> {
    match timeout(config.connect_timeout, client.refresh_tokens()).await {
        Ok(r) => r,
        Err(_) => Err(AnyListError::AuthenticationFailed(format!(
            "Token refresh timed out after {:?}",
            config.connect_timeout
        ))),
    }
}

/// The first connection: report its result to `connect()`. A 401 refreshes
/// the tokens once and retries.
async fn first_connect(client: &AnyListClient, config: &RealtimeConfig) -> Result<WsStream> {
    match open(client, config).await {
        Ok(ws) => Ok(ws),
        Err(ConnectError::Failed(msg)) => Err(AnyListError::NetworkError(msg)),
        Err(ConnectError::Unauthorized) => {
            refresh(client, config).await.map_err(|e| match e {
                AnyListError::AuthenticationFailed(_) => e,
                other => AnyListError::AuthenticationFailed(other.to_string()),
            })?;
            match open(client, config).await {
                Ok(ws) => Ok(ws),
                Err(ConnectError::Unauthorized) => Err(AnyListError::NetworkError(
                    "WebSocket handshake rejected with 401 after token refresh".to_string(),
                )),
                Err(ConnectError::Failed(msg)) => Err(AnyListError::NetworkError(msg)),
            }
        }
    }
}

/// How a session ended.
enum SessionEnd {
    /// The consumer closed the connection.
    Closed,
    /// The connection was lost (server close, stream error or end, liveness
    /// timeout, write failure). `refresh` is set for close code 4010.
    Lost { refresh: bool },
    /// `reconnect()` was called.
    Requested,
}

/// Run `fut` while watching the command channel. Returns `None` if a `Close`
/// arrives (or the handle is gone), which drops `fut`. `Reconnect` is
/// ignored, since an attempt is already in flight.
async fn unless_closed<F: Future>(
    cmd_rx: &mut mpsc::UnboundedReceiver<Command>,
    fut: F,
) -> Option<F::Output> {
    tokio::pin!(fut);
    loop {
        tokio::select! {
            out = &mut fut => return Some(out),
            cmd = cmd_rx.recv() => match cmd {
                Some(Command::Reconnect) => continue,
                Some(Command::Close) | None => return None,
            },
        }
    }
}

fn normal_close() -> Option<CloseFrame<'static>> {
    Some(CloseFrame {
        code: CloseCode::Normal,
        reason: "".into(),
    })
}

struct ActorContext {
    client: Arc<AnyListClient>,
    config: RealtimeConfig,
    callback: SyncCallback,
    state: StateTx,
}

impl ActorContext {
    async fn run(self, ws: WsStream, mut cmd_rx: mpsc::UnboundedReceiver<Command>) {
        let mut backoff = ReconnectionStrategy::with_delays(
            self.config.reconnect_initial_delay,
            self.config.reconnect_max_delay,
        );
        let mut ws = ws;
        let mut first = true;
        loop {
            if !first {
                set_state(&self.state, ConnectionState::Connected);
                (self.callback)(SyncEvent::Reconnected);
            }
            first = false;

            let started = Instant::now();
            let (refresh, skip_delay) = match self.session(ws, &mut cmd_rx).await {
                SessionEnd::Closed => break,
                SessionEnd::Lost { refresh } => (refresh, false),
                SessionEnd::Requested => (false, true),
            };
            // Only a session that lived for a while resets the backoff, so a
            // server that accepts and immediately drops is not hammered.
            if started.elapsed() >= self.config.liveness_timeout {
                backoff.reset();
            }
            set_state(&self.state, ConnectionState::Reconnecting);

            match self
                .reestablish(&mut backoff, &mut cmd_rx, refresh, skip_delay)
                .await
            {
                Some(new_ws) => ws = new_ws,
                None => break,
            }
        }
        set_state(&self.state, ConnectionState::Closed);
    }

    /// One WebSocket session.
    async fn session(
        &self,
        mut ws: WsStream,
        cmd_rx: &mut mpsc::UnboundedReceiver<Command>,
    ) -> SessionEnd {
        let interval = self.config.heartbeat_interval;
        let mut ticker = tokio::time::interval_at(Instant::now() + interval, interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut last_seen = Instant::now();

        loop {
            tokio::select! {
                msg = ws.next() => match msg {
                    Some(Ok(msg)) => {
                        // Any inbound frame proves the connection is alive.
                        last_seen = Instant::now();
                        match msg {
                            Message::Text(text) => {
                                if let Some(event) = SyncEvent::from_message(&text) {
                                    if event != SyncEvent::Heartbeat {
                                        (self.callback)(event);
                                    }
                                }
                            }
                            Message::Close(frame) => {
                                let code = frame.map(|f| u16::from(f.code));
                                // Flush the close reply tungstenite queued.
                                let reply = ws.send(Message::Close(None));
                                let _ = timeout(self.config.send_timeout, reply).await;
                                return SessionEnd::Lost {
                                    refresh: code == Some(4010),
                                };
                            }
                            _ => {}
                        }
                    }
                    // Stream error, or end of stream after a close handshake.
                    Some(Err(_)) | None => return SessionEnd::Lost { refresh: false },
                },
                _ = ticker.tick() => {
                    if last_seen.elapsed() > self.config.liveness_timeout {
                        self.abandon(ws).await;
                        return SessionEnd::Lost { refresh: false };
                    }
                    if !self.send_heartbeat(&mut ws).await {
                        return SessionEnd::Lost { refresh: false };
                    }
                }
                cmd = cmd_rx.recv() => match cmd {
                    Some(Command::Reconnect) => {
                        self.abandon(ws).await;
                        return SessionEnd::Requested;
                    }
                    Some(Command::Close) | None => {
                        self.close_gracefully(ws).await;
                        return SessionEnd::Closed;
                    }
                },
            }
        }
    }

    /// Write `--heartbeat--` and a Ping (flushed together) under
    /// `send_timeout`. The Ping gives liveness even if the server does not
    /// echo the text heartbeat, since any Pong counts as an inbound frame.
    async fn send_heartbeat(&self, ws: &mut WsStream) -> bool {
        let write = async {
            ws.feed(Message::Text(HEARTBEAT.to_string())).await?;
            ws.feed(Message::Ping(Vec::new())).await?;
            ws.flush().await
        };
        matches!(timeout(self.config.send_timeout, write).await, Ok(Ok(())))
    }

    /// Best-effort close frame, then drop the socket without waiting for the
    /// peer.
    async fn abandon(&self, mut ws: WsStream) {
        let _ = timeout(
            self.config.send_timeout,
            ws.send(Message::Close(normal_close())),
        )
        .await;
    }

    /// Close frame, then wait for the peer's reply or EOF, all within
    /// `close_timeout`. Frames that arrive meanwhile are not dispatched.
    async fn close_gracefully(&self, mut ws: WsStream) {
        let _ = timeout(self.config.close_timeout, async {
            if ws.send(Message::Close(normal_close())).await.is_err() {
                return;
            }
            while let Some(Ok(_)) = ws.next().await {}
        })
        .await;
    }

    /// Back off, refresh tokens if needed, and attempt to reconnect until it
    /// works. Returns `None` if closed meanwhile.
    async fn reestablish(
        &self,
        backoff: &mut ReconnectionStrategy,
        cmd_rx: &mut mpsc::UnboundedReceiver<Command>,
        mut needs_refresh: bool,
        mut skip_delay: bool,
    ) -> Option<WsStream> {
        loop {
            if !skip_delay {
                let delay = backoff.next_delay();
                if !delay.is_zero() {
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {}
                        cmd = cmd_rx.recv() => match cmd {
                            // reconnect(): skip the rest of the delay.
                            Some(Command::Reconnect) => {}
                            Some(Command::Close) | None => return None,
                        },
                    }
                }
            }
            skip_delay = false;

            if needs_refresh {
                // Refresh only for close 4010 or a 401 upgrade, and never
                // connect with a token known to be stale.
                match unless_closed(cmd_rx, refresh(&self.client, &self.config)).await? {
                    Ok(()) => needs_refresh = false,
                    Err(_) => continue,
                }
            }

            match unless_closed(cmd_rx, open(&self.client, &self.config)).await? {
                Ok(ws) => return Some(ws),
                Err(ConnectError::Unauthorized) => needs_refresh = true,
                Err(ConnectError::Failed(_)) => {}
            }
        }
    }
}
