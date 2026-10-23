//! A scriptable fake AnyList server for the realtime tests.
//!
//! One `127.0.0.1:0` listener serves both the WebSocket endpoint
//! (`/data/add-user-listener`) and the token refresh endpoint
//! (`/auth/token/refresh`). The HTTP side is parsed by hand so that the fake
//! can reject an upgrade with any status, hang a handshake, or drop to raw TCP
//! (no automatic Pong) after the upgrade.
//!
//! Every WebSocket upgrade request is an *attempt* and gets a [`Conn`] record,
//! whatever its outcome. Each attempt takes the next [`Behaviour`] from the
//! script queue, or the default behaviour once the queue is empty.

#![allow(dead_code)]

use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Role};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub const HEARTBEAT: &str = "--heartbeat--";

/// How the fake answers the upgrade request.
#[derive(Clone, Debug)]
pub enum Handshake {
    Accept,
    /// Accept after a delay (keeps the client in `Reconnecting` long enough
    /// for a 5 ms poller to see it, and orders accepts after earlier ends).
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
    /// Drop the TCP socket without a close frame.
    DropTcp,
    /// Stop answering: read and discard raw bytes, never write, keep TCP open.
    GoQuiet,
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
    /// Drop TCP as soon as any frame arrives from the client.
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
    /// Read error (for example a reset).
    Error(String),
    /// The fake dropped the socket itself.
    ServerDrop,
}

/// One WebSocket upgrade attempt.
#[derive(Clone, Debug)]
pub struct Conn {
    pub index: usize,
    pub tcp_accepted_at: Instant,
    pub request_at: Instant,
    pub path: String,
    pub query: String,
    pub outcome: Outcome,
    pub upgraded_at: Option<Instant>,
    pub frames: Vec<(Instant, Frame)>,
    /// When the fake sent its own close frame.
    pub server_close_at: Option<Instant>,
    /// When the fake went quiet.
    pub quiet_at: Option<Instant>,
    pub ended_at: Option<Instant>,
    pub end: Option<End>,
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

#[derive(Default)]
struct Inner {
    conns: Mutex<Vec<Conn>>,
    controls: Mutex<Vec<Option<mpsc::UnboundedSender<Out>>>>,
    script: Mutex<VecDeque<Behaviour>>,
    default: Mutex<Option<Behaviour>>,
    refresh_script: Mutex<VecDeque<RefreshReply>>,
    refreshes: Mutex<Vec<Refresh>>,
    open: AtomicUsize,
    max_open: AtomicUsize,
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

pub struct FakeServer {
    pub addr: SocketAddr,
    inner: Arc<Inner>,
    accept_task: JoinHandle<()>,
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.accept_task.abort();
        for t in self.inner.tasks.lock().unwrap().drain(..) {
            t.abort();
        }
    }
}

impl FakeServer {
    /// Start on an ephemeral port with the given default behaviour.
    pub async fn start(default: Behaviour) -> Self {
        Self::start_on(0, default).await
    }

    /// Start on a given port (0 for ephemeral).
    pub async fn start_on(port: u16, default: Behaviour) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let inner = Arc::new(Inner::default());
        *inner.default.lock().unwrap() = Some(default);
        let inner2 = Arc::clone(&inner);
        let accept_task = tokio::spawn(async move {
            loop {
                let Ok((tcp, _)) = listener.accept().await else {
                    continue;
                };
                let accepted_at = Instant::now();
                let inner3 = Arc::clone(&inner2);
                let task = tokio::spawn(handle_tcp(inner3, tcp, accepted_at));
                inner2.tasks.lock().unwrap().push(task);
            }
        });
        Self {
            addr,
            inner,
            accept_task,
        }
    }

    pub fn ws_url(&self) -> String {
        format!("ws://{}", self.addr)
    }
    pub fn http_url(&self) -> String {
        format!("http://{}", self.addr)
    }
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Queue behaviours for the next attempts, in order.
    pub fn script(&self, behaviours: impl IntoIterator<Item = Behaviour>) {
        self.inner.script.lock().unwrap().extend(behaviours);
    }
    /// Replace the behaviour used once the script queue is empty.
    pub fn set_default(&self, b: Behaviour) {
        *self.inner.default.lock().unwrap() = Some(b);
    }
    /// Queue replies for refresh requests (unscripted requests get 500).
    pub fn script_refresh(&self, replies: impl IntoIterator<Item = RefreshReply>) {
        self.inner.refresh_script.lock().unwrap().extend(replies);
    }

    /// All upgrade attempts so far.
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
    pub fn concurrent_open(&self) -> usize {
        self.inner.open.load(Ordering::SeqCst)
    }
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

    /// Wait until attempt `i` exists (any outcome).
    pub async fn wait_attempt(&self, i: usize, within: Duration) -> Option<Conn> {
        let end = Instant::now() + within;
        loop {
            if let Some(c) = self.conns().get(i) {
                return Some(c.clone());
            }
            if Instant::now() >= end {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
    /// Wait until attempt `i` has been upgraded.
    pub async fn wait_upgraded(&self, i: usize, within: Duration) -> Option<Conn> {
        let end = Instant::now() + within;
        loop {
            if let Some(c) = self.conns().get(i) {
                if c.upgraded() {
                    return Some(c.clone());
                }
            }
            if Instant::now() >= end {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
    /// Wait until attempt `i` has ended.
    pub async fn wait_ended(&self, i: usize, within: Duration) -> Option<Conn> {
        let end = Instant::now() + within;
        loop {
            if let Some(c) = self.conns().get(i) {
                if c.ended_at.is_some() {
                    return Some(c.clone());
                }
            }
            if Instant::now() >= end {
                return None;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }

    fn update(&self, i: usize, f: impl FnOnce(&mut Conn)) {
        update(&self.inner, i, f)
    }
}

fn update(inner: &Inner, i: usize, f: impl FnOnce(&mut Conn)) {
    if let Some(c) = inner.conns.lock().unwrap().get_mut(i) {
        f(c);
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

async fn read_request(tcp: &mut TcpStream) -> Option<Request> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(p) = find(&buf, b"\r\n\r\n") {
            break p + 4;
        }
        let n = tcp.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?;
    let target = request_line.split(' ').nth(1)?.to_string();
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.clone(), String::new()),
    };
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((k.trim().to_ascii_lowercase(), v.trim().to_string()))
        })
        .collect();
    let mut body = buf[header_end..].to_vec();
    let req = Request {
        path,
        query,
        headers,
        body: Vec::new(),
    };
    if let Some(len) = req
        .header("content-length")
        .and_then(|v| v.parse::<usize>().ok())
    {
        while body.len() < len {
            let n = tcp.read(&mut chunk).await.ok()?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..n]);
        }
    } else if req
        .header("transfer-encoding")
        .is_some_and(|v| v.contains("chunked"))
    {
        while find(&body, b"0\r\n\r\n").is_none() {
            let n = tcp.read(&mut chunk).await.ok()?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..n]);
        }
    }
    Some(Request { body, ..req })
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

async fn handle_tcp(inner: Arc<Inner>, mut tcp: TcpStream, tcp_accepted_at: Instant) {
    let Some(req) = read_request(&mut tcp).await else {
        return;
    };
    let request_at = Instant::now();

    if req.path.starts_with("/auth/token/refresh") {
        handle_refresh(&inner, tcp, req, request_at).await;
        return;
    }

    let behaviour = {
        let next = inner.script.lock().unwrap().pop_front();
        next.unwrap_or_else(|| inner.default.lock().unwrap().clone().unwrap())
    };
    let index = {
        let mut conns = inner.conns.lock().unwrap();
        let index = conns.len();
        conns.push(Conn {
            index,
            tcp_accepted_at,
            request_at,
            path: req.path.clone(),
            query: req.query.clone(),
            outcome: Outcome::Pending,
            upgraded_at: None,
            frames: Vec::new(),
            server_close_at: None,
            quiet_at: None,
            ended_at: None,
            end: None,
        });
        inner.controls.lock().unwrap().push(None);
        index
    };

    match behaviour.handshake {
        Handshake::Reject(status) => {
            let resp = format!(
                "HTTP/1.1 {status} Rejected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = tcp.write_all(resp.as_bytes()).await;
            let _ = tcp.flush().await;
            update(&inner, index, |c| {
                c.outcome = Outcome::Rejected(status);
                c.ended_at = Some(Instant::now());
                c.end = Some(End::ServerDrop);
            });
            return;
        }
        Handshake::Hang => {
            update(&inner, index, |c| c.outcome = Outcome::Hung);
            let end = read_until_eof(&mut tcp).await;
            update(&inner, index, |c| {
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
    if tcp.write_all(resp.as_bytes()).await.is_err() {
        update(&inner, index, |c| {
            c.ended_at = Some(Instant::now());
            c.end = Some(End::Error("101 write failed".into()));
        });
        return;
    }
    let open = inner.open.fetch_add(1, Ordering::SeqCst) + 1;
    inner.max_open.fetch_max(open, Ordering::SeqCst);
    update(&inner, index, |c| {
        c.outcome = Outcome::Upgraded;
        c.upgraded_at = Some(Instant::now());
    });

    let (tx, rx) = mpsc::unbounded_channel();
    inner.controls.lock().unwrap()[index] = Some(tx.clone());
    let steps = tokio::spawn(run_steps(behaviour.steps.clone(), tx));

    let end = if behaviour.raw {
        run_raw(&inner, index, tcp, rx).await
    } else {
        let ws = WebSocketStream::from_raw_socket(tcp, Role::Server, None).await;
        run_ws(&inner, index, ws, rx, &behaviour).await
    };
    steps.abort();
    // Record the end before the open count drops, so a reader that sees
    // the count fall also sees the end time.
    update(&inner, index, |c| {
        c.ended_at = Some(Instant::now());
        c.end = Some(end);
    });
    inner.open.fetch_sub(1, Ordering::SeqCst);
}

async fn run_steps(steps: Vec<Step>, tx: mpsc::UnboundedSender<Out>) {
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

async fn read_until_eof(tcp: &mut TcpStream) -> End {
    let mut buf = [0u8; 4096];
    loop {
        match tcp.read(&mut buf).await {
            Ok(0) => return End::Eof,
            Ok(_) => {}
            Err(e) => return End::Error(e.to_string()),
        }
    }
}

fn record(inner: &Inner, index: usize, frame: Frame) {
    update(inner, index, |c| c.frames.push((Instant::now(), frame)));
}

async fn run_ws(
    inner: &Inner,
    index: usize,
    mut ws: WebSocketStream<TcpStream>,
    mut rx: mpsc::UnboundedReceiver<Out>,
    behaviour: &Behaviour,
) -> End {
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
                    update(inner, index, |c| c.server_close_at = Some(Instant::now()));
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

async fn go_quiet(inner: &Inner, index: usize, mut ws: WebSocketStream<TcpStream>) -> End {
    update(inner, index, |c| c.quiet_at = Some(Instant::now()));
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

async fn run_raw(
    inner: &Inner,
    index: usize,
    mut tcp: TcpStream,
    mut rx: mpsc::UnboundedReceiver<Out>,
) -> End {
    let mut buf = [0u8; 4096];
    let mut quiet = false;
    loop {
        tokio::select! {
            r = tcp.read(&mut buf) => match r {
                Ok(0) => return End::Eof,
                Ok(_) => {}
                Err(e) => return End::Error(e.to_string()),
            },
            out = rx.recv(), if !quiet => match out {
                Some(Out::Send(Message::Text(t))) => {
                    let _ = tcp.write_all(&raw_frame(0x1, t.as_bytes())).await;
                }
                Some(Out::Send(Message::Binary(b))) => {
                    let _ = tcp.write_all(&raw_frame(0x2, &b)).await;
                }
                Some(Out::Send(Message::Ping(p))) => {
                    let _ = tcp.write_all(&raw_frame(0x9, &p)).await;
                }
                Some(Out::Send(_)) => {}
                Some(Out::Close(code)) => {
                    update(inner, index, |c| c.server_close_at = Some(Instant::now()));
                    let _ = tcp.write_all(&raw_frame(0x8, &code.to_be_bytes())).await;
                }
                Some(Out::DropTcp) => return End::ServerDrop,
                Some(Out::GoQuiet) => {
                    update(inner, index, |c| c.quiet_at = Some(Instant::now()));
                    quiet = true;
                }
                None => quiet = true,
            },
        }
    }
}

async fn handle_refresh(inner: &Inner, mut tcp: TcpStream, req: Request, at: Instant) {
    inner.refreshes.lock().unwrap().push(Refresh {
        at,
        headers: req.headers.clone(),
        body: String::from_utf8_lossy(&req.body).to_string(),
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
    let _ = tcp.write_all(resp.as_bytes()).await;
    let _ = tcp.flush().await;
}
