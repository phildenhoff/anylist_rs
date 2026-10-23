//! Black-box tests for `anylist_rs::realtime`, one per scenario of the
//! issue #24 behaviour spec (`RT-<AREA>-<nn>` becomes `rt_<area>_<nn>_...`).
//!
//! Every test talks to a scriptable fake AnyList server on 127.0.0.1 (see
//! `fake.rs`) through the public API only, with millisecond timings from
//! `RealtimeConfig` (heartbeat 50 ms, liveness 150 ms, backoff 100 ms
//! doubling to 400 ms, connect/close/send timeouts 500 ms).

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
