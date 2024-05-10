use std::sync;
use std::time;

use csv;
use serde_derive::Serialize;

// enum OutputType {
//     Default,
//     Json,
//     Csv,
// }

pub struct State {
    //output: OutputType,
    start: time::Instant,
    requested: usize,
    processed: usize,
    success: usize,
    fail: usize,
    error: usize,
    killed: bool,
    mux: sync::Mutex<()>,
}

#[derive(Serialize)]
pub struct StateSerialize {
    took: u128,
    requested: usize,
    processed: usize,
    success: usize,
    fail: usize,
    error: usize,
}

impl State {
    pub fn new(r: usize) -> Self {
        //pub fn new(r: usize, output: String) -> Self {
        // let mut t_output = OutputType::Default;

        // // Tried making this a match and it didn't work, advice?
        // if output == "json".to_string() {
        //     t_output = OutputType::Json;
        // } else if output == "csv".to_string() {
        //     t_output = OutputType::Csv;
        // }

        State {
            start: time::Instant::now(),
            requested: r,
            processed: 0,
            success: 0,
            fail: 0,
            error: 0,
            killed: false,
            mux: sync::Mutex::new(()),
            // output: t_output,
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

    fn to_seralizer(&self) -> StateSerialize {
        let took = time::Instant::now() - self.start;
        return StateSerialize {
            took: took.as_millis(),
            requested: self.requested,
            processed: self.processed,
            success: self.success,
            fail: self.fail,
            error: self.error,
        };
    }

    pub fn to_json(&self) -> String {
        return serde_json::to_string(&self.to_seralizer()).expect("failed to seralize json");
    }

    pub fn to_csv(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut wtrb = csv::WriterBuilder::new();
        wtrb.has_headers(true);
        let mut wtr = wtrb.from_writer(vec![]);
        wtr.serialize(&self.to_seralizer())?;
        return Ok(String::from_utf8(wtr.into_inner()?)?);
    }
}

#[test]
fn increment_test() {
    //let mut state = State::new(4, "default".to_string());
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
    //let mut state = State::new(1, "default".to_string());
    let mut state = State::new(1);
    assert!(!state.done());
    state.increment(1, 0, 0);
    assert!(state.done());
}

#[test]
fn string_test() {
    let expected = String::from("requested=4 processed=1 success=1 fail=0 error=0 duration=");
    //let mut state = State::new(4, "default".to_string());
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
