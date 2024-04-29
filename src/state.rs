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

#[test]
fn increment_test() {
    let mut state = State::new(4);
    state.increment(1, 0, 0);
    state.increment(0, 1, 0);
    state.increment(0, 0, 1);
    assert_eq!(state.success, 1);
    assert_eq!(state.fail, 1);
    assert_eq!(state.error, 1);
    assert_eq!(state.processed, 3);
}

#[test]
fn done_test() {
    let mut state = State::new(1);
    assert!(!state.done());
    state.increment(1, 0, 0);
    assert!(state.done());
}

#[test]
fn string_test() {
    let expected = String::from("requested=4 processed=1 success=1 fail=0 error=0 duration=");
    let mut state = State::new(4);
    state.increment(1, 0, 0);

    let got = state.string();
    assert!(
        got.contains(&expected),
        "expected '{}' to contain '{}'",
        got,
        expected
    );
}
