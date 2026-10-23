//! Black-box tests for `anylist_rs::realtime`, one per scenario of the
//! issue #24 behaviour spec (`RT-<AREA>-<nn>` becomes `rt_<area>_<nn>_...`).
//!
//! Every test talks to a scriptable fake AnyList server (see `fake.rs`)
//! through the public API, plus the `#[doc(hidden)]` transport seam
//! (`RealtimeConfig::with_transport`) that runs the WebSocket over in-memory
//! pipes.
//!
//! Most tests run on tokio's paused clock (`start_paused = true`) with the
//! production timings (heartbeat 5 s, liveness 15 s, backoff 2 s doubling
//! to 120 s, connect timeout 10 s, close and send timeouts 5 s). The clock
//! only moves when every task is idle, so a run is a pure function of the
//! code under test: timing assertions are exact, minutes of protocol time
//! take milliseconds, and CPU load cannot make a test fail.
//!
//! A few tests run in real time (`flavor = "multi_thread"`), each marked
//! with the reason: token refresh goes over real HTTP (`reqwest` cannot use
//! an in-memory pipe), RT-EV-07 needs a callback that really blocks its
//! thread, and RT-CN-07 checks the default TCP path. Their timing
//! assertions only use causal ordering or margins of hundreds of
//! milliseconds.

mod fake;
mod support;

mod cl;
mod cn;
mod dc;
mod dr;
mod ev;
mod hb;
mod issue24;
mod lv;
mod rc;
