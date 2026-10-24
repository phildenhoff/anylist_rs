//! A scriptable fake AnyList server for the realtime tests.
//!
//! The WebSocket side runs in memory: [`FakeServer::transport`] is plugged
//! into `RealtimeConfig::with_transport`, and every connection attempt the
//! client makes becomes a `tokio::io::duplex` pipe whose far end is served
//! by a task of this fake. No byte goes through the kernel, so the tests can
//! run on tokio's paused clock: the clock only moves when every task is
//! idle, and timings come out exact.
//!
//! The token refresh endpoint (`/auth/token/refresh`) is real HTTP, because
//! the client sends it with `reqwest`, which cannot run over an in-memory
//! pipe. It is served by a plain OS thread on `127.0.0.1:0`, so it works the
//! same whatever the test's runtime or clock. Tests that expect no refresh
//! use it as a tripwire: it counts every TCP connection it accepts.
//!
//! [`FakeServer::start_tcp`] also serves the WebSocket endpoint on a real
//! TCP listener, for the one test that goes through the default
//! `connect_async` path.
//!
//! Every connection attempt gets a [`Conn`] record, whatever its outcome,
//! and takes the next [`Behaviour`] from the script queue, or the default
//! behaviour once the queue is empty. The HTTP side of the upgrade is parsed
//! by hand so that the fake can reject an upgrade with any status, hang a
//! handshake, or drop to raw bytes (no automatic Pong) after the upgrade.

#![allow(dead_code)]

use anylist_rs::realtime::config::{RealtimeStream, TransportFuture};
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, DuplexStream, ReadBuf};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Role};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub const HEARTBEAT: &str = "--heartbeat--";

/// Capacity of each in-memory pipe, per direction. The fake always keeps
/// reading, so this is never the limit in practice.
const PIPE_BYTES: usize = 256 * 1024;

/// How the fake answers the upgrade request.
#[derive(Clone, Debug)]
pub enum Handshake {
    Accept,
    /// Accept after a delay (keeps the client in `Reconnecting` long enough
    /// for a poller to see it).
    AcceptAfter(Duration),
    /// Answer with this HTTP status instead of 101.
    Reject(u16),
    /// Read the request, never answer, wait for the client to go away.
    Hang,
}

/// Scripted server actions, run in order after the upgrade.
#[derive(Clone, Debug)]
pub enum Step {
    Sleep(Duration),
    Text(String),
    Binary(Vec<u8>),
    Ping,
    /// Send a close frame with this code, then keep reading until EOF.
    Close(u16),
    /// Drop the connection without a close frame.
    DropTcp,
    /// Stop answering: read and discard raw bytes, never write, keep the
    /// connection open.
    GoQuiet,
    /// From now on every write the *client* makes on this connection fails
    /// (broken pipe), while its reads keep working.
    BreakClientWrites,
    /// Send this text immediately and then every period, forever.
    TextEvery(String, Duration),
    /// Send a Ping immediately and then every period, forever.
    PingEvery(Duration),
}

#[derive(Clone, Debug)]
pub struct Behaviour {
    pub handshake: Handshake,
    /// Echo every `--heartbeat--` back.
    pub echo: bool,
    /// After this many echoes, go quiet (half-open).
    pub echo_limit: Option<usize>,
    /// Raw mode: after the upgrade, read and discard bytes without WebSocket
    /// processing (so no automatic Pong). Steps are written as raw frames.
    pub raw: bool,
    /// On receipt of the client's close frame, go quiet instead of replying.
    pub ignore_close: bool,
    /// Drop the connection as soon as any frame arrives from the client.
    pub drop_on_inbound: bool,
    pub steps: Vec<Step>,
}

impl Behaviour {
    /// Accept, read everything, never send text. The WebSocket library
    /// still answers Pings with Pongs.
    pub fn silent() -> Self {
        Self {
            handshake: Handshake::Accept,
            echo: false,
            echo_limit: None,
            raw: false,
            ignore_close: false,
            drop_on_inbound: false,
            steps: Vec::new(),
        }
    }
    /// Accept and echo heartbeats.
    pub fn echo() -> Self {
        Self {
            echo: true,
            ..Self::silent()
        }
    }
    /// Accept, then read and discard raw bytes: no echo, no Pong.
    pub fn raw() -> Self {
        Self {
            raw: true,
            ..Self::silent()
        }
    }
    pub fn reject(status: u16) -> Self {
        Self {
            handshake: Handshake::Reject(status),
            ..Self::silent()
        }
    }
    pub fn hang() -> Self {
        Self {
            handshake: Handshake::Hang,
            ..Self::silent()
        }
    }
    pub fn delay_handshake(mut self, d: Duration) -> Self {
        self.handshake = Handshake::AcceptAfter(d);
        self
    }
    pub fn steps(mut self, steps: Vec<Step>) -> Self {
        self.steps = steps;
        self
    }
    pub fn echo_limit(mut self, n: usize) -> Self {
        self.echo_limit = Some(n);
        self
    }
    pub fn ignore_close(mut self) -> Self {
        self.ignore_close = true;
        self
    }
    pub fn drop_on_inbound(mut self) -> Self {
        self.drop_on_inbound = true;
        self
    }
}

/// A frame received from the client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Text(String),
    Binary(usize),
    Ping,
    Pong,
    Close(Option<u16>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Upgraded,
    Rejected(u16),
    Hung,
}

/// How a connection ended, as seen by the fake.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum End {
    /// Read EOF or the library reported the close handshake as complete.
    Eof,
    /// Read error.
    Error(String),
    /// The fake dropped the connection itself.
    ServerDrop,
}

/// One connection attempt.
///
/// `seq` fields come from one counter per fake, so they order events that
/// happen at the same (virtual) instant.
#[derive(Clone, Debug)]
pub struct Conn {
    pub index: usize,
    /// When the client opened the connection.
    pub accepted_at: Instant,
    pub accepted_seq: u64,
    /// When the fake had read the upgrade request.
    pub request_at: Option<Instant>,
    pub path: String,
    pub query: String,
    pub outcome: Outcome,
    pub upgraded_at: Option<Instant>,
    pub frames: Vec<(Instant, Frame)>,
    /// When the fake sent its own close frame.
    pub server_close_at: Option<Instant>,
    /// When the fake went quiet.
    pub quiet_at: Option<Instant>,
    /// When the fake saw the connection end.
    pub ended_at: Option<Instant>,
    pub end: Option<End>,
    /// When the client dropped its end of the pipe (in-memory only).
    pub client_closed_at: Option<Instant>,
    pub client_closed_seq: Option<u64>,
}

impl Conn {
    pub fn param(&self, key: &str) -> Option<String> {
        self.query.split('&').find_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            if k == key {
                Some(urlencoding::decode(v).ok()?.into_owned())
            } else {
                None
            }
        })
    }
    pub fn access_token(&self) -> Option<String> {
        self.param("access_token")
    }
    pub fn request_at(&self) -> Instant {
        self.request_at.expect("upgrade request not read yet")
    }
    pub fn heartbeats(&self) -> Vec<Instant> {
        self.frames
            .iter()
            .filter(|(_, f)| *f == Frame::Text(HEARTBEAT.to_string()))
            .map(|(t, _)| *t)
            .collect()
    }
    pub fn pings(&self) -> Vec<Instant> {
        self.frames
            .iter()
            .filter(|(_, f)| *f == Frame::Ping)
            .map(|(t, _)| *t)
            .collect()
    }
    pub fn close_frames(&self) -> Vec<Option<u16>> {
        self.frames
            .iter()
            .filter_map(|(_, f)| match f {
                Frame::Close(c) => Some(*c),
                _ => None,
            })
            .collect()
    }
    pub fn upgraded(&self) -> bool {
        self.upgraded_at.is_some()
    }
}

/// A request received on `/auth/token/refresh`.
#[derive(Clone, Debug)]
pub struct Refresh {
    pub at: Instant,
    /// Header names are lowercased.
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Refresh {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == &name.to_ascii_lowercase())
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Clone, Debug)]
pub enum RefreshReply {
    Tokens(String, String),
    Status(u16),
}

#[derive(Debug)]
enum Out {
    Send(Message),
    Close(u16),
    DropTcp,
    GoQuiet,
}

struct Inner {
    conns: Mutex<Vec<Conn>>,
    controls: Mutex<Vec<Option<mpsc::UnboundedSender<Out>>>>,
    script: Mutex<VecDeque<Behaviour>>,
    default: Mutex<Behaviour>,
    refresh_script: Mutex<VecDeque<RefreshReply>>,
    refreshes: Mutex<Vec<Refresh>>,
    /// TCP connections accepted by the refresh endpoint.
    refresh_connections: AtomicUsize,
    /// In-memory connections refused while `refusing`.
    refusing: AtomicBool,
    refused: AtomicUsize,
    /// Client ends of in-memory connections currently alive.
    open: AtomicUsize,
    max_open: AtomicUsize,
    seq: AtomicU64,
    /// Bumped on every change, so waiters need not poll.
    version: watch::Sender<u64>,
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl Inner {
    fn new(default: Behaviour) -> Self {
        Self {
            conns: Mutex::new(Vec::new()),
            controls: Mutex::new(Vec::new()),
            script: Mutex::new(VecDeque::new()),
            default: Mutex::new(default),
            refresh_script: Mutex::new(VecDeque::new()),
            refreshes: Mutex::new(Vec::new()),
            refresh_connections: AtomicUsize::new(0),
            refusing: AtomicBool::new(false),
            refused: AtomicUsize::new(0),
            open: AtomicUsize::new(0),
            max_open: AtomicUsize::new(0),
            seq: AtomicU64::new(0),
            version: watch::channel(0).0,
            tasks: Mutex::new(Vec::new()),
        }
    }

    fn bump(&self) {
        self.version.send_modify(|v| *v += 1);
    }

    fn next_seq(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::SeqCst)
    }

    fn update(&self, i: usize, f: impl FnOnce(&mut Conn)) {
        if let Some(c) = self.conns.lock().unwrap().get_mut(i) {
            f(c);
        }
        self.bump();
    }

    /// Record a new attempt and pick its behaviour.
    fn new_conn(&self) -> (usize, Behaviour) {
        let behaviour = {
            let next = self.script.lock().unwrap().pop_front();
            next.unwrap_or_else(|| self.default.lock().unwrap().clone())
        };
        let index = {
            let mut conns = self.conns.lock().unwrap();
            let index = conns.len();
            conns.push(Conn {
                index,
                accepted_at: Instant::now(),
                accepted_seq: self.next_seq(),
                request_at: None,
                path: String::new(),
                query: String::new(),
                outcome: Outcome::Pending,
                upgraded_at: None,
                frames: Vec::new(),
                server_close_at: None,
                quiet_at: None,
                ended_at: None,
                end: None,
                client_closed_at: None,
                client_closed_seq: None,
            });
            self.controls.lock().unwrap().push(None);
            index
        };
        self.bump();
        (index, behaviour)
    }

    fn spawn(&self, fut: impl std::future::Future<Output = ()> + Send + 'static) {
        let task = tokio::spawn(fut);
        self.tasks.lock().unwrap().push(task);
    }
}

/// The client's end of an in-memory connection. Records when the client
/// drops it, and can be made to fail every write.
struct ClientEnd {
    io: DuplexStream,
    index: usize,
    inner: Arc<Inner>,
    broken: Arc<AtomicBool>,
}

impl AsyncRead for ClientEnd {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.io).poll_read(cx, buf)
    }
}

fn broken_pipe() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "fake: client writes broken")
}

impl AsyncWrite for ClientEnd {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.broken.load(Ordering::SeqCst) {
            return Poll::Ready(Err(broken_pipe()));
        }
        Pin::new(&mut self.io).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.broken.load(Ordering::SeqCst) {
            return Poll::Ready(Err(broken_pipe()));
        }
        Pin::new(&mut self.io).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.io).poll_shutdown(cx)
    }
}

impl Drop for ClientEnd {
    fn drop(&mut self) {
        let seq = self.inner.next_seq();
        self.inner.update(self.index, |c| {
            c.client_closed_at = Some(Instant::now());
            c.client_closed_seq = Some(seq);
        });
        self.inner.open.fetch_sub(1, Ordering::SeqCst);
    }
}

/// The HTTP refresh endpoint, on an OS thread.
struct RefreshServer {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
}

impl RefreshServer {
    fn start(inner: Arc<Inner>) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = Arc::clone(&stop);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if stop2.load(Ordering::SeqCst) {
                    return;
                }
                let Ok(stream) = stream else { continue };
                inner.refresh_connections.fetch_add(1, Ordering::SeqCst);
                inner.bump();
                serve_refresh(&inner, stream);
            }
        });
        Self { addr, stop }
    }
}

impl Drop for RefreshServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wake the accept loop so the thread exits.
        let _ = std::net::TcpStream::connect(self.addr);
    }
}

pub struct FakeServer {
    inner: Arc<Inner>,
    refresh: RefreshServer,
    tcp: Option<(SocketAddr, JoinHandle<()>)>,
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        if let Some((_, task)) = &self.tcp {
            task.abort();
        }
        for t in self.inner.tasks.lock().unwrap().drain(..) {
            t.abort();
        }
    }
}

impl FakeServer {
    /// An in-memory fake with the given default behaviour. Connect to it
    /// through [`FakeServer::transport`].
    pub fn start(default: Behaviour) -> Self {
        let inner = Arc::new(Inner::new(default));
        let refresh = RefreshServer::start(Arc::clone(&inner));
        Self {
            inner,
            refresh,
            tcp: None,
        }
    }

    /// Also serve the WebSocket endpoint on a real TCP listener at
    /// `127.0.0.1:0` (see [`FakeServer::tcp_http_url`]).
    pub async fn start_tcp(default: Behaviour) -> Self {
        let mut server = Self::start(default);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let inner = Arc::clone(&server.inner);
        let task = tokio::spawn(async move {
            loop {
                let Ok((tcp, _)) = listener.accept().await else {
                    continue;
                };
                let (index, behaviour) = inner.new_conn();
                let inner2 = Arc::clone(&inner);
                inner.spawn(handle_conn(
                    inner2,
                    index,
                    tcp,
                    behaviour,
                    Arc::new(AtomicBool::new(false)),
                ));
            }
        });
        server.tcp = Some((addr, task));
        server
    }

    /// A connector for `RealtimeConfig::with_transport`: each call is one
    /// connection attempt to this fake.
    pub fn transport(&self) -> impl Fn(&str) -> TransportFuture + Send + Sync + 'static {
        let inner = Arc::clone(&self.inner);
        move |_url: &str| {
            let inner = Arc::clone(&inner);
            Box::pin(async move {
                if inner.refusing.load(Ordering::SeqCst) {
                    inner.refused.fetch_add(1, Ordering::SeqCst);
                    inner.bump();
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionRefused,
                        "fake: connection refused",
                    ));
                }
                let (client, server) = tokio::io::duplex(PIPE_BYTES);
                let open = inner.open.fetch_add(1, Ordering::SeqCst) + 1;
                inner.max_open.fetch_max(open, Ordering::SeqCst);
                let (index, behaviour) = inner.new_conn();
                let broken = Arc::new(AtomicBool::new(false));
                inner.spawn(handle_conn(
                    Arc::clone(&inner),
                    index,
                    server,
                    behaviour,
                    Arc::clone(&broken),
                ));
                let end = ClientEnd {
                    io: client,
                    index,
                    inner,
                    broken,
                };
                Ok(Box::new(end) as Box<dyn RealtimeStream>)
            }) as TransportFuture
        }
    }

    /// Base URL of the refresh endpoint (real HTTP on 127.0.0.1).
    pub fn http_url(&self) -> String {
        format!("http://{}", self.refresh.addr)
    }
    /// Base URL of the TCP WebSocket listener ([`FakeServer::start_tcp`]).
    pub fn tcp_http_url(&self) -> String {
        format!("http://{}", self.tcp.as_ref().expect("not a TCP fake").0)
    }

    /// While set, in-memory connection attempts fail with
    /// `ConnectionRefused` (nothing listening).
    pub fn refuse(&self, on: bool) {
        self.inner.refusing.store(on, Ordering::SeqCst);
    }
    pub fn refused(&self) -> usize {
        self.inner.refused.load(Ordering::SeqCst)
    }
    /// Every connection attempt the client made, refused or not.
    pub fn connects(&self) -> usize {
        self.refused() + self.attempts()
    }

    /// Queue behaviours for the next attempts, in order.
    pub fn script(&self, behaviours: impl IntoIterator<Item = Behaviour>) {
        self.inner.script.lock().unwrap().extend(behaviours);
    }
    /// Replace the behaviour used once the script queue is empty.
    pub fn set_default(&self, b: Behaviour) {
        *self.inner.default.lock().unwrap() = b;
    }
    /// Queue replies for refresh requests (unscripted requests get 500).
    pub fn script_refresh(&self, replies: impl IntoIterator<Item = RefreshReply>) {
        self.inner.refresh_script.lock().unwrap().extend(replies);
    }

    /// All connection attempts so far.
    pub fn conns(&self) -> Vec<Conn> {
        self.inner.conns.lock().unwrap().clone()
    }
    pub fn conn(&self, i: usize) -> Conn {
        self.conns()[i].clone()
    }
    pub fn attempts(&self) -> usize {
        self.inner.conns.lock().unwrap().len()
    }
    pub fn upgraded(&self) -> Vec<Conn> {
        self.conns().into_iter().filter(|c| c.upgraded()).collect()
    }
    pub fn refreshes(&self) -> Vec<Refresh> {
        self.inner.refreshes.lock().unwrap().clone()
    }
    /// TCP connections accepted by the refresh endpoint (a refresh attempt
    /// shows up here even if it never completes).
    pub fn refresh_connections(&self) -> usize {
        self.inner.refresh_connections.load(Ordering::SeqCst)
    }
    /// Client ends of in-memory connections alive right now.
    pub fn concurrent_open(&self) -> usize {
        self.inner.open.load(Ordering::SeqCst)
    }
    /// The most client ends ever alive at once: 1 means each connection was
    /// dropped by the client before the next one was opened.
    pub fn max_concurrent_open(&self) -> usize {
        self.inner.max_open.load(Ordering::SeqCst)
    }

    fn control(&self, i: usize, out: Out) {
        if let Some(Some(tx)) = self.inner.controls.lock().unwrap().get(i) {
            let _ = tx.send(out);
        }
    }
    pub fn send_text(&self, i: usize, text: &str) {
        self.control(i, Out::Send(Message::Text(text.to_string())));
    }
    pub fn send_binary(&self, i: usize, data: Vec<u8>) {
        self.control(i, Out::Send(Message::Binary(data)));
    }
    pub fn close(&self, i: usize, code: u16) {
        self.control(i, Out::Close(code));
    }
    pub fn drop_tcp(&self, i: usize) {
        self.control(i, Out::DropTcp);
    }
    pub fn go_quiet(&self, i: usize) {
        self.control(i, Out::GoQuiet);
    }

    /// Wait (without polling) until `f` returns `Some`, for at most `within`.
    pub async fn wait_for<T>(
        &self,
        within: Duration,
        mut f: impl FnMut(&[Conn]) -> Option<T>,
    ) -> Option<T> {
        let mut rx = self.inner.version.subscribe();
        let deadline = Instant::now() + within;
        loop {
            if let Some(t) = f(&self.inner.conns.lock().unwrap()) {
                return Some(t);
            }
            if tokio::time::timeout_at(deadline, rx.changed())
                .await
                .is_err()
            {
                return None;
            }
        }
    }
    /// Wait until attempt `i` exists (any outcome).
    pub async fn wait_attempt(&self, i: usize, within: Duration) -> Option<Conn> {
        self.wait_for(within, |c| c.get(i).cloned()).await
    }
    /// Wait until attempt `i` has had its upgrade request read.
    pub async fn wait_request(&self, i: usize, within: Duration) -> Option<Conn> {
        self.wait_for(within, |c| {
            c.get(i).filter(|c| c.request_at.is_some()).cloned()
        })
        .await
    }
    /// Wait until attempt `i` has been upgraded.
    pub async fn wait_upgraded(&self, i: usize, within: Duration) -> Option<Conn> {
        self.wait_for(within, |c| c.get(i).filter(|c| c.upgraded()).cloned())
            .await
    }
    /// Wait until the fake has seen attempt `i` end.
    pub async fn wait_ended(&self, i: usize, within: Duration) -> Option<Conn> {
        self.wait_for(within, |c| {
            c.get(i).filter(|c| c.ended_at.is_some()).cloned()
        })
        .await
    }
    /// Wait until `n` refresh requests have been answered.
    pub async fn wait_refreshes(&self, n: usize, within: Duration) -> bool {
        let inner = Arc::clone(&self.inner);
        self.wait_for(within, move |_| {
            (inner.refreshes.lock().unwrap().len() >= n).then_some(())
        })
        .await
        .is_some()
    }
}

struct Request {
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn parse_head(head: &[u8]) -> Option<Request> {
    let head = String::from_utf8_lossy(head).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?;
    let target = request_line.split(' ').nth(1)?.to_string();
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let headers = lines
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((k.trim().to_ascii_lowercase(), v.trim().to_string()))
        })
        .collect();
    Some(Request {
        path,
        query,
        headers,
        body: Vec::new(),
    })
}

/// Read an upgrade request (headers only; an upgrade has no body).
async fn read_upgrade<S: AsyncRead + Unpin>(io: &mut S) -> Option<Request> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(p) = find(&buf, b"\r\n\r\n") {
            break p + 4;
        }
        let n = io.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    parse_head(&buf[..header_end])
}

async fn handle_conn<S>(
    inner: Arc<Inner>,
    index: usize,
    mut io: S,
    behaviour: Behaviour,
    broken: Arc<AtomicBool>,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let Some(req) = read_upgrade(&mut io).await else {
        inner.update(index, |c| {
            c.ended_at = Some(Instant::now());
            c.end = Some(End::Eof);
        });
        return;
    };
    inner.update(index, |c| {
        c.request_at = Some(Instant::now());
        c.path = req.path.clone();
        c.query = req.query.clone();
    });

    match behaviour.handshake {
        Handshake::Reject(status) => {
            let resp = format!(
                "HTTP/1.1 {status} Rejected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = io.write_all(resp.as_bytes()).await;
            let _ = io.flush().await;
            inner.update(index, |c| {
                c.outcome = Outcome::Rejected(status);
                c.ended_at = Some(Instant::now());
                c.end = Some(End::ServerDrop);
            });
            return;
        }
        Handshake::Hang => {
            inner.update(index, |c| c.outcome = Outcome::Hung);
            let end = read_until_eof(&mut io).await;
            inner.update(index, |c| {
                c.ended_at = Some(Instant::now());
                c.end = Some(end);
            });
            return;
        }
        Handshake::AcceptAfter(d) => tokio::time::sleep(d).await,
        Handshake::Accept => {}
    }

    let key = req.header("sec-websocket-key").unwrap_or("").to_string();
    let resp = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
        derive_accept_key(key.as_bytes())
    );
    if io.write_all(resp.as_bytes()).await.is_err() {
        inner.update(index, |c| {
            c.ended_at = Some(Instant::now());
            c.end = Some(End::Error("101 write failed".into()));
        });
        return;
    }
    inner.update(index, |c| {
        c.outcome = Outcome::Upgraded;
        c.upgraded_at = Some(Instant::now());
    });

    let (tx, rx) = mpsc::unbounded_channel();
    inner.controls.lock().unwrap()[index] = Some(tx.clone());
    let steps = tokio::spawn(run_steps(behaviour.steps.clone(), tx, broken));

    let end = if behaviour.raw {
        run_raw(&inner, index, io, rx).await
    } else {
        let ws = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        run_ws(&inner, index, ws, rx, &behaviour).await
    };
    steps.abort();
    inner.update(index, |c| {
        c.ended_at = Some(Instant::now());
        c.end = Some(end);
    });
}

async fn run_steps(steps: Vec<Step>, tx: mpsc::UnboundedSender<Out>, broken: Arc<AtomicBool>) {
    for step in steps {
        match step {
            Step::Sleep(d) => tokio::time::sleep(d).await,
            Step::Text(t) => {
                let _ = tx.send(Out::Send(Message::Text(t)));
            }
            Step::Binary(b) => {
                let _ = tx.send(Out::Send(Message::Binary(b)));
            }
            Step::Ping => {
                let _ = tx.send(Out::Send(Message::Ping(Vec::new())));
            }
            Step::Close(code) => {
                let _ = tx.send(Out::Close(code));
            }
            Step::DropTcp => {
                let _ = tx.send(Out::DropTcp);
            }
            Step::GoQuiet => {
                let _ = tx.send(Out::GoQuiet);
            }
            Step::BreakClientWrites => broken.store(true, Ordering::SeqCst),
            Step::TextEvery(t, every) => loop {
                if tx.send(Out::Send(Message::Text(t.clone()))).is_err() {
                    return;
                }
                tokio::time::sleep(every).await;
            },
            Step::PingEvery(every) => loop {
                if tx.send(Out::Send(Message::Ping(Vec::new()))).is_err() {
                    return;
                }
                tokio::time::sleep(every).await;
            },
        }
    }
}

async fn read_until_eof<S: AsyncRead + Unpin>(io: &mut S) -> End {
    let mut buf = [0u8; 4096];
    loop {
        match io.read(&mut buf).await {
            Ok(0) => return End::Eof,
            Ok(_) => {}
            Err(e) => return End::Error(e.to_string()),
        }
    }
}

fn record(inner: &Inner, index: usize, frame: Frame) {
    inner.update(index, |c| c.frames.push((Instant::now(), frame)));
}

async fn run_ws<S>(
    inner: &Inner,
    index: usize,
    mut ws: WebSocketStream<S>,
    mut rx: mpsc::UnboundedReceiver<Out>,
    behaviour: &Behaviour,
) -> End
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut echoes = 0usize;
    loop {
        tokio::select! {
            msg = ws.next() => match msg {
                Some(Ok(m)) => {
                    let frame = match &m {
                        Message::Text(t) => Frame::Text(t.clone()),
                        Message::Binary(b) => Frame::Binary(b.len()),
                        Message::Ping(_) => Frame::Ping,
                        Message::Pong(_) => Frame::Pong,
                        Message::Close(f) => Frame::Close(f.as_ref().map(|f| u16::from(f.code))),
                        Message::Frame(_) => continue,
                    };
                    record(inner, index, frame.clone());
                    if behaviour.drop_on_inbound {
                        return End::ServerDrop;
                    }
                    match frame {
                        Frame::Text(t) if t == HEARTBEAT && behaviour.echo => {
                            let _ = ws.send(Message::Text(HEARTBEAT.to_string())).await;
                            echoes += 1;
                            if behaviour.echo_limit == Some(echoes) {
                                return go_quiet(inner, index, ws).await;
                            }
                        }
                        Frame::Close(_) if behaviour.ignore_close => {
                            return go_quiet(inner, index, ws).await;
                        }
                        _ => {}
                    }
                }
                Some(Err(e)) => return End::Error(e.to_string()),
                None => return End::Eof,
            },
            out = rx.recv() => match out {
                Some(Out::Send(m)) => {
                    let _ = ws.send(m).await;
                }
                Some(Out::Close(code)) => {
                    inner.update(index, |c| c.server_close_at = Some(Instant::now()));
                    let _ = ws
                        .close(Some(CloseFrame {
                            code: CloseCode::from(code),
                            reason: "".into(),
                        }))
                        .await;
                }
                Some(Out::DropTcp) => return End::ServerDrop,
                Some(Out::GoQuiet) => return go_quiet(inner, index, ws).await,
                None => {}
            },
        }
    }
}

async fn go_quiet<S: AsyncRead + AsyncWrite + Unpin>(
    inner: &Inner,
    index: usize,
    mut ws: WebSocketStream<S>,
) -> End {
    inner.update(index, |c| c.quiet_at = Some(Instant::now()));
    read_until_eof(ws.get_mut()).await
}

/// Encode an unmasked server-to-client frame.
fn raw_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0x80 | opcode];
    let len = payload.len();
    if len < 126 {
        out.push(len as u8);
    } else if len <= u16::MAX as usize {
        out.push(126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    out.extend_from_slice(payload);
    out
}

async fn run_raw<S: AsyncRead + AsyncWrite + Unpin>(
    inner: &Inner,
    index: usize,
    mut io: S,
    mut rx: mpsc::UnboundedReceiver<Out>,
) -> End {
    let mut buf = [0u8; 4096];
    let mut quiet = false;
    loop {
        tokio::select! {
            r = io.read(&mut buf) => match r {
                Ok(0) => return End::Eof,
                Ok(_) => {}
                Err(e) => return End::Error(e.to_string()),
            },
            out = rx.recv(), if !quiet => match out {
                Some(Out::Send(Message::Text(t))) => {
                    let _ = io.write_all(&raw_frame(0x1, t.as_bytes())).await;
                }
                Some(Out::Send(Message::Binary(b))) => {
                    let _ = io.write_all(&raw_frame(0x2, &b)).await;
                }
                Some(Out::Send(Message::Ping(p))) => {
                    let _ = io.write_all(&raw_frame(0x9, &p)).await;
                }
                Some(Out::Send(_)) => {}
                Some(Out::Close(code)) => {
                    inner.update(index, |c| c.server_close_at = Some(Instant::now()));
                    let _ = io.write_all(&raw_frame(0x8, &code.to_be_bytes())).await;
                }
                Some(Out::DropTcp) => return End::ServerDrop,
                Some(Out::GoQuiet) => {
                    inner.update(index, |c| c.quiet_at = Some(Instant::now()));
                    quiet = true;
                }
                None => quiet = true,
            },
        }
    }
}

/// Serve one refresh request on the refresh thread (blocking I/O).
fn serve_refresh(inner: &Inner, mut tcp: std::net::TcpStream) {
    let _ = tcp.set_read_timeout(Some(Duration::from_secs(10)));
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(p) = find(&buf, b"\r\n\r\n") {
            break p + 4;
        }
        match tcp.read(&mut chunk) {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    };
    let Some(req) = parse_head(&buf[..header_end]) else {
        return;
    };
    if !req.path.starts_with("/auth/token/refresh") {
        return;
    }
    let mut body = buf[header_end..].to_vec();
    if let Some(len) = req
        .header("content-length")
        .and_then(|v| v.parse::<usize>().ok())
    {
        while body.len() < len {
            match tcp.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => body.extend_from_slice(&chunk[..n]),
            }
        }
    } else if req
        .header("transfer-encoding")
        .is_some_and(|v| v.contains("chunked"))
    {
        while find(&body, b"0\r\n\r\n").is_none() {
            match tcp.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => body.extend_from_slice(&chunk[..n]),
            }
        }
    }
    inner.refreshes.lock().unwrap().push(Refresh {
        at: Instant::now(),
        headers: req.headers.clone(),
        body: String::from_utf8_lossy(&body).to_string(),
    });
    let reply = inner
        .refresh_script
        .lock()
        .unwrap()
        .pop_front()
        .unwrap_or(RefreshReply::Status(500));
    let (status, body) = match reply {
        RefreshReply::Tokens(a, r) => (
            200,
            format!(r#"{{"access_token":"{a}","refresh_token":"{r}"}}"#),
        ),
        RefreshReply::Status(s) => (s, r#"{"error":"scripted"}"#.to_string()),
    };
    let resp = format!(
        "HTTP/1.1 {status} Scripted\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = tcp.write_all(resp.as_bytes());
    let _ = tcp.flush();
    inner.bump();
}
