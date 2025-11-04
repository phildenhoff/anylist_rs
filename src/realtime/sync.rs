use crate::client::AnyListClient;
use crate::error::{AnyListError, Result};
use crate::realtime::events::SyncEvent;
use crate::realtime::heartbeat::HeartbeatManager;
use crate::realtime::reconnect::ReconnectionStrategy;
use crate::realtime::SyncCallback;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};

/// Connection state for real-time sync
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connecting to server
    Connecting,
    /// Connected and active
    Connected,
    /// Connection closed, will retry
    Reconnecting,
    /// Permanently closed
    Closed,
}

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Real-time sync manager
pub struct RealtimeSync {
    /// Reference to the AnyList client
    client: Arc<AnyListClient>,

    /// Current connection state
    state: Arc<Mutex<ConnectionState>>,

    /// Callback for sync events
    callback: SyncCallback,

    /// WebSocket stream (if connected)
    ws_stream: Arc<Mutex<Option<WsStream>>>,

    /// Background tasks handles
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,

    /// Shutdown signal
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl RealtimeSync {
    /// Create a new real-time sync instance, without connecting
    ///
    /// Call `connect()` to establish the WebSocket connection.
    pub fn new<F>(client: Arc<AnyListClient>, callback: F) -> Self
    where
        F: Fn(SyncEvent) + Send + Sync + 'static,
    {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

        Self {
            client,
            state: Arc::new(Mutex::new(ConnectionState::Disconnected)),
            callback: Arc::new(callback),
            ws_stream: Arc::new(Mutex::new(None)),
            tasks: Arc::new(Mutex::new(Vec::new())),
            shutdown_tx,
        }
    }

    /// Connect to the WebSocket server.
    ///
    /// Idempotent (returns `Ok()`), unless previously closed via
    /// `disconnect()`.
    pub async fn connect(&mut self) -> Result<()> {
        // Validate state transition
        {
            let mut state = self.state.lock().await;
            match *state {
                ConnectionState::Connected => {
                    // Already connected, nothing to do
                    return Ok(());
                }
                ConnectionState::Connecting => {
                    // Already connecting, nothing to do
                    return Ok(());
                }
                ConnectionState::Closed => {
                    return Err(AnyListError::NetworkError(
                        "Cannot connect after permanent closure".to_string(),
                    ));
                }
                _ => {
                    *state = ConnectionState::Connecting;
                }
            }
        }

        let url = self.build_websocket_url().await?;

        let (ws_stream, _response) = connect_async(&url).await.map_err(|e| {
            // Reset state on connection failure
            let state = self.state.clone();
            tokio::spawn(async move {
                let mut s = state.lock().await;
                *s = ConnectionState::Disconnected;
            });
            AnyListError::NetworkError(format!("WebSocket connection failed: {}", e))
        })?;

        // Store stream
        {
            let mut stream_guard = self.ws_stream.lock().await;
            *stream_guard = Some(ws_stream);
        }

        // Update state
        {
            let mut state = self.state.lock().await;
            *state = ConnectionState::Connected;
        }

        self.start_background_tasks().await;

        Ok(())
    }

    /// Build the WebSocket URL with auth parameters
    async fn build_websocket_url(&self) -> Result<String> {
        let tokens = self.client.export_tokens()?;

        let url = format!(
            "wss://www.anylist.com/data/add-user-listener?client_id={}&access_token={}",
            urlencoding::encode(self.client.client_identifier()),
            urlencoding::encode(&tokens.access_token)
        );

        Ok(url)
    }

    pub async fn state(&self) -> ConnectionState {
        *self.state.lock().await
    }

    pub async fn is_connected(&self) -> bool {
        *self.state.lock().await == ConnectionState::Connected
    }

    /// Start background tasks (message receiver, heartbeat sender, reconnection monitor)
    async fn start_background_tasks(&mut self) {
        let mut tasks = self.tasks.lock().await;

        let heartbeat_mgr = Arc::new(HeartbeatManager::new());

        let receiver_task = self.spawn_message_receiver(Arc::clone(&heartbeat_mgr));
        tasks.push(receiver_task);

        let sender_task = self.spawn_heartbeat_sender(Arc::clone(&heartbeat_mgr));
        tasks.push(sender_task);

        let reconnect_task = self.spawn_reconnection_monitor();
        tasks.push(reconnect_task);
    }

    /// Receive next message from WebSocket stream
    async fn recv_message(ws_stream: &Arc<Mutex<Option<WsStream>>>) -> Option<std::result::Result<Message, tokio_tungstenite::tungstenite::Error>> {
        let mut stream_guard = ws_stream.lock().await;
        if let Some(stream) = stream_guard.as_mut() {
            stream.next().await
        } else {
            None
        }
    }

    /// Handle text message by parsing and dispatching event
    async fn handle_text_message(
        text: &str,
        heartbeat_mgr: &Arc<HeartbeatManager>,
        callback: &SyncCallback,
    ) {
        if let Some(event) = SyncEvent::from_message(text) {
            if event == SyncEvent::Heartbeat {
                // Reset heartbeat counter
                heartbeat_mgr.reset().await;
            } else {
                // Dispatch non-heartbeat events to callback
                callback(event);
            }
        }
    }

    /// Handle WebSocket close event and update connection state
    async fn handle_close(
        frame: Option<tokio_tungstenite::tungstenite::protocol::CloseFrame<'_>>,
        state: &Arc<Mutex<ConnectionState>>,
    ) {
        let code = frame.as_ref().map(|f| f.code.into());
        let mut state_guard = state.lock().await;
        match code {
            Some(1000) => {
                // Normal closure
                *state_guard = ConnectionState::Disconnected;
            }
            Some(4010) => {
                // Token expired - need to refresh
                *state_guard = ConnectionState::Reconnecting;
            }
            _ => {
                // Abnormal closure - will reconnect
                *state_guard = ConnectionState::Reconnecting;
            }
        }
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        msg_result: std::result::Result<Message, tokio_tungstenite::tungstenite::Error>,
        heartbeat_mgr: &Arc<HeartbeatManager>,
        callback: &SyncCallback,
        state: &Arc<Mutex<ConnectionState>>,
    ) -> bool {
        match msg_result {
            Ok(Message::Text(text)) => {
                Self::handle_text_message(&text, heartbeat_mgr, callback).await;
                true // Continue loop
            }
            Ok(Message::Close(frame)) => {
                Self::handle_close(frame, state).await;
                false // Exit loop
            }
            Err(_e) => {
                // WebSocket error - trigger reconnection
                let mut state_guard = state.lock().await;
                *state_guard = ConnectionState::Reconnecting;
                false // Exit loop
            }
            _ => {
                // Ignore binary, ping, pong messages
                true // Continue loop
            }
        }
    }

    fn spawn_message_receiver(&self, heartbeat_mgr: Arc<HeartbeatManager>) -> JoinHandle<()> {
        let ws_stream = Arc::clone(&self.ws_stream);
        let callback = Arc::clone(&self.callback);
        let state = Arc::clone(&self.state);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    msg_result = Self::recv_message(&ws_stream) => {
                        if let Some(msg_result) = msg_result {
                            let should_continue = Self::handle_message(
                                msg_result,
                                &heartbeat_mgr,
                                &callback,
                                &state,
                            ).await;

                            if !should_continue {
                                break;
                            }
                        } else {
                            // Stream is None, exit
                            break;
                        }
                    }
                }
            }
        })
    }

    /// Check if too many heartbeats have been missed
    /// Returns true if we should continue, false if we should reconnect
    async fn check_heartbeat_timeout(
        heartbeat_mgr: &Arc<HeartbeatManager>,
        state: &Arc<Mutex<ConnectionState>>,
    ) -> bool {
        let missed = heartbeat_mgr.increment_missed().await;

        if missed >= 3 {
            // Too many missed heartbeats, trigger reconnection
            let mut state_guard = state.lock().await;
            *state_guard = ConnectionState::Reconnecting;
            false
        } else {
            true
        }
    }

    /// Send heartbeat message over WebSocket
    /// Returns true if we should continue, false if we should stop
    async fn send_heartbeat(
        ws_stream: &Arc<Mutex<Option<WsStream>>>,
        state: &Arc<Mutex<ConnectionState>>,
    ) -> bool {
        let mut stream_guard = ws_stream.lock().await;

        if let Some(stream) = stream_guard.as_mut() {
            if let Err(_e) = stream.send(Message::Text("--heartbeat--".to_string())).await {
                drop(stream_guard);
                let mut state_guard = state.lock().await;
                *state_guard = ConnectionState::Reconnecting;
                false
            } else {
                true
            }
        } else {
            // No stream available
            false
        }
    }

    fn spawn_heartbeat_sender(&self, heartbeat_mgr: Arc<HeartbeatManager>) -> JoinHandle<()> {
        let ws_stream = Arc::clone(&self.ws_stream);
        let state = Arc::clone(&self.state);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut ticker = interval(heartbeat_mgr.interval());
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = ticker.tick() => {
                        // Check if connection is still alive
                        let current_state = *state.lock().await;
                        if current_state != ConnectionState::Connected {
                            break;
                        }

                        // Check for heartbeat timeout
                        if !Self::check_heartbeat_timeout(&heartbeat_mgr, &state).await {
                            break;
                        }

                        // Send heartbeat message
                        if !Self::send_heartbeat(&ws_stream, &state).await {
                            break;
                        }
                    }
                }
            }
        })
    }

    fn spawn_reconnection_monitor(&self) -> JoinHandle<()> {
        let state = Arc::clone(&self.state);
        let ws_stream = Arc::clone(&self.ws_stream);
        let client = Arc::clone(&self.client);
        let callback = Arc::clone(&self.callback);
        let shutdown_tx = self.shutdown_tx.clone();
        let tasks = Arc::clone(&self.tasks);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut strategy = ReconnectionStrategy::new();

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        let current_state = *state.lock().await;

                        if current_state == ConnectionState::Reconnecting {
                            let delay = strategy.next_delay();

                            tokio::time::sleep(delay).await;

                            match Self::attempt_reconnect(
                                Arc::clone(&client),
                                Arc::clone(&state),
                                Arc::clone(&ws_stream),
                                Arc::clone(&callback),
                                shutdown_tx.clone(),
                                Arc::clone(&tasks),
                            ).await {
                                Ok(_) => {
                                    strategy.reset();
                                    let mut state_guard = state.lock().await;
                                    *state_guard = ConnectionState::Connected;
                                }
                                Err(_e) => {
                                    // Will retry on next loop iteration
                                }
                            }
                        } else if current_state == ConnectionState::Connected {
                            // Reset backoff on successful connection
                            strategy.reset();
                        } else if current_state == ConnectionState::Closed {
                            // Permanently closed, exit
                            break;
                        }
                    }
                }
            }
        })
    }

    /// Attempt to reconnect (used by reconnection monitor)
    async fn attempt_reconnect(
        client: Arc<AnyListClient>,
        state: Arc<Mutex<ConnectionState>>,
        ws_stream: Arc<Mutex<Option<WsStream>>>,
        callback: SyncCallback,
        shutdown_tx: tokio::sync::broadcast::Sender<()>,
        tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    ) -> Result<()> {
        // Try to refresh tokens first (in case token expired)
        match client.refresh_tokens().await {
            Ok(_) => {}
            Err(_) => {
                // Continue anyway - maybe the token is still valid
            }
        }

        // Build URL
        let tokens = client.export_tokens()?;
        let url = format!(
            "wss://www.anylist.com/data/add-user-listener?client_id={}&access_token={}",
            urlencoding::encode(client.client_identifier()),
            urlencoding::encode(&tokens.access_token)
        );

        // Connect
        let (new_stream, _) = connect_async(&url)
            .await
            .map_err(|e| AnyListError::NetworkError(format!("Reconnection failed: {}", e)))?;

        // Replace stream
        {
            let mut stream_guard = ws_stream.lock().await;
            *stream_guard = Some(new_stream);
        }

        // Update state
        {
            let mut state_guard = state.lock().await;
            *state_guard = ConnectionState::Connected;
        }

        // Restart background tasks (heartbeat sender and message receiver)
        let heartbeat_mgr = Arc::new(HeartbeatManager::new());

        // Spawn new message receiver
        let receiver_task = {
            let ws_stream = Arc::clone(&ws_stream);
            let callback = Arc::clone(&callback);
            let state = Arc::clone(&state);
            let mut shutdown_rx = shutdown_tx.subscribe();
            let heartbeat_mgr = Arc::clone(&heartbeat_mgr);

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            break;
                        }
                        msg_result = async {
                            let mut stream_guard = ws_stream.lock().await;
                            if let Some(stream) = stream_guard.as_mut() {
                                stream.next().await
                            } else {
                                None
                            }
                        } => {
                            if let Some(msg_result) = msg_result {
                                match msg_result {
                                    Ok(Message::Text(text)) => {
                                        if let Some(event) = SyncEvent::from_message(&text) {
                                            if event == SyncEvent::Heartbeat {
                                                heartbeat_mgr.reset().await;
                                            } else {
                                                callback(event);
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        let mut state_guard = state.lock().await;
                                        *state_guard = ConnectionState::Reconnecting;
                                        break;
                                    }
                                    Err(_) => {
                                        let mut state_guard = state.lock().await;
                                        *state_guard = ConnectionState::Reconnecting;
                                        break;
                                    }
                                    _ => {}
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            })
        };

        // Spawn new heartbeat sender
        let sender_task = {
            let ws_stream = Arc::clone(&ws_stream);
            let state = Arc::clone(&state);
            let mut shutdown_rx = shutdown_tx.subscribe();

            tokio::spawn(async move {
                let mut ticker = interval(heartbeat_mgr.interval());
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            break;
                        }
                        _ = ticker.tick() => {
                            let current_state = *state.lock().await;
                            if current_state != ConnectionState::Connected {
                                break;
                            }

                            let missed = heartbeat_mgr.increment_missed().await;
                            if missed >= 3 {
                                let mut state_guard = state.lock().await;
                                *state_guard = ConnectionState::Reconnecting;
                                break;
                            }

                            let mut stream_guard = ws_stream.lock().await;
                            if let Some(stream) = stream_guard.as_mut() {
                                if (stream.send(Message::Text("--heartbeat--".to_string())).await).is_err() {
                                    drop(stream_guard);
                                    let mut state_guard = state.lock().await;
                                    *state_guard = ConnectionState::Reconnecting;
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            })
        };

        // Add tasks to the list
        {
            let mut tasks_guard = tasks.lock().await;
            tasks_guard.push(receiver_task);
            tasks_guard.push(sender_task);
        }

        Ok(())
    }

    /// Disconnect and stop all background tasks
    pub async fn disconnect(&mut self) -> Result<()> {
        // Update state
        {
            let mut state = self.state.lock().await;
            *state = ConnectionState::Closed;
        }

        // Send shutdown signal to all tasks
        let _ = self.shutdown_tx.send(());

        // Close WebSocket with normal closure code
        {
            let mut stream_guard = self.ws_stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                let _ = stream.close(None).await;
            }
            *stream_guard = None;
        }

        // Wait for all tasks to complete
        let mut tasks = self.tasks.lock().await;
        for task in tasks.drain(..) {
            let _ = task.await;
        }

        Ok(())
    }

    /// Manually trigger reconnection
    pub async fn reconnect(&mut self) -> Result<()> {
        // Check if already connected
        {
            let state = self.state.lock().await;
            if *state == ConnectionState::Connected {
                return Ok(());
            }
            if *state == ConnectionState::Closed {
                return Err(AnyListError::NetworkError(
                    "Cannot reconnect after permanent closure".to_string(),
                ));
            }
        }

        // Trigger reconnection by setting state
        // The reconnection monitor will pick this up
        {
            let mut state = self.state.lock().await;
            *state = ConnectionState::Reconnecting;
        }

        Ok(())
    }
}

impl Drop for RealtimeSync {
    fn drop(&mut self) {
        // Best effort cleanup - can't use async in Drop
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());
    }
}
