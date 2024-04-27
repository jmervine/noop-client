use std::sync;
use std::time;

pub struct State {
    start: time::Instant,
    requested: usize,
    processed: usize,
    success: usize,
    fail: usize,
    error: usize,
    killed: bool,
    mux: sync::Mutex<()>,
}

impl State {
    pub fn new(r: usize) -> Self {
        State {
            start: time::Instant::now(),
            requested: r,
            processed: 0,
            success: 0,
            fail: 0,
            error: 0,
            killed: false,
            mux: sync::Mutex::new(()),
        }
    }

    pub fn increment(&mut self, success: usize, fail: usize, error: usize) {
        let _lock = self.mux.lock();
        self.processed += 1;
        self.success += success;
        self.fail += fail;
        self.error += error;
    }

    pub fn done(&self) -> bool {
        let _lock = self.mux.lock();
        self.killed || self.requested == self.processed
    }

    pub fn kill(&mut self) {
        let _lock = self.mux.lock();
        self.killed = true;
    }

    pub fn string(&self) -> String {
        let _lock = self.mux.lock();
        let duration = time::Instant::now() - self.start;
        return format!(
            "requested={} processed={} success={} fail={} error={} duration={:?}",
            self.requested, self.processed, self.success, self.fail, self.error, duration,
        );
    }
}
