use itertools::Itertools;
use serde::Serialize;
use std::io::Write;
use std::fs::OpenOptions;
use std::time::Duration;
use std::sync::Arc;

use crate::bernoulli::ManyCategoryBernEstimator;
use crate::bernoulli::ManyCategoryBernSummary;
// use crate::normal::ManyCategoryNormalEstimator;
// use crate::normal::ManyCategoryNormalSummary;
use crate::markov::ContextMarkovEstimator;
use crate::markov::ContextMarkovSummary;
use crate::record::SpanRecord;


const INITIAL_STATE: &str = "__INITIAL_STATE__";
const SUCCESS_STATE: &str = "__SUCCESS_STATE__";
const FAILURE_STATE: &str = "__FAILURE_STATE__";
const TOTAL_SUBRECORD: &str = "__TOTAL__";
const WRITE_PERIOD: Duration = Duration::from_secs(10);
const DEPENDENCY_OUT: &str = "dependency_summary.jsons";

#[derive(Serialize, Debug)]
pub struct DependencySummary {
    span_markov: ContextMarkovSummary<String>,
    fail_bernoulli: ManyCategoryBernSummary<String>,
    // time_normal: ManyCategoryNormalSummary<String>,
}

impl DependencySummary {
    pub fn make_span_markov(&self) -> ContextMarkovSummary<String> {
        // TODO: make model out of summary
        self.span_markov.clone()
    }

    pub fn make_fail_bernoulli(&self) -> ManyCategoryBernSummary<String> {
        // TODO: make model out of summary
        self.fail_bernoulli.clone()
    }

    // pub fn make_time_normal(&self) -> ManyCategoryNormalSummary<String> {
    //     // TODO: make model out of summary
    //     self.time_normal.clone()
    // }
}

/* Process dependency data */
pub struct DependencyProcessor {
    span_markov: ContextMarkovEstimator<String>,
    fail_bernoulli: ManyCategoryBernEstimator<String>,
    // time_normal: ManyCategoryNormalEstimator<String>,
}

impl Default for DependencyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyProcessor {
    pub fn new() -> Self {
        DependencyProcessor {
            span_markov: ContextMarkovEstimator::default(),
            fail_bernoulli: ManyCategoryBernEstimator::default(),
            // time_normal: ManyCategoryNormalEstimator::default(),
        }
    }

    pub fn record_span(&self, current_sr: &SpanRecord, parent_sr: &SpanRecord) {
        log::trace!("Span {} [ {} ]", self.map_record(parent_sr), self.map_record(current_sr));
        self.span_markov.observe(
            self.map_record(parent_sr),
            INITIAL_STATE.to_string(),
            self.map_record(current_sr),
        );
        // self.time_normal.observe(
        //     self.map_record(parent_sr),
        //     self.map_record(current_sr),
        //     parent_sr.elapsed().as_secs_f64(),
        // );
    }

    pub fn record_span_follows(&self, current_sr: &SpanRecord, prev_sr: &SpanRecord, parent_sr: &SpanRecord) {
        log::trace!("Span {} [ {} --> {} ]", self.map_record(parent_sr), self.map_record(prev_sr), self.map_record(current_sr));
        self.span_markov.observe(
            self.map_record(parent_sr),
            self.map_record(prev_sr),
            self.map_record(current_sr),
        );
        // self.time_normal.observe(
        //     self.map_record(parent_sr),
        //     self.map_record(current_sr),
        //     parent_sr.elapsed().as_secs_f64(),
        // );
    }

    pub fn record_span_succeeds(&self, parent_sr: &SpanRecord) {
        self.record_span_succeeds_inner(INITIAL_STATE.to_string(), parent_sr);
    }

    pub fn record_span_succeeds_from(&self, current_sr: &SpanRecord, parent_sr: &SpanRecord) {
        self.record_span_succeeds_inner(self.map_record(current_sr), parent_sr);
    }

    pub fn record_span_fails(&self, parent_sr: &SpanRecord) {
        self.record_span_fails_inner(INITIAL_STATE.to_string(), parent_sr);
    }

    pub fn record_span_fails_from(&self, current_sr: &SpanRecord, parent_sr: &SpanRecord) {
        self.record_span_fails_inner(self.map_record(current_sr), parent_sr);
    }

    pub fn summarize(&self) -> DependencySummary {
        DependencySummary {
            span_markov: self.span_markov.clone().summarize(),
            fail_bernoulli: self.fail_bernoulli.clone().summarize(),
            // time_normal: self.time_normal.clone().summarize(),
        }
    }

    pub fn write_summary(&self) -> std::io::Result<()> {
        let summary = self.summarize();
        log::debug!("Current dependency summary {:#?}", summary);

        if let Ok(summary_json) = serde_json::to_string(&summary) {
            if let Ok(mut log_file) = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(DEPENDENCY_OUT) {
                log_file.write_all(summary_json.as_bytes())?;
                log_file.write_all(b"\n")?;
                log::info!(
                    "Wrote dependency {} characters to {}",
                    summary_json.len(),
                    DEPENDENCY_OUT
                );
            } else {
                log::error!("Failed to write dependency summary");
            }
        } else {
            log::error!("Failed to serialize dependency summary");
        }
        Ok(())
    }

    pub fn install_periodic_write_threaded(self: Arc<Self>) {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(WRITE_PERIOD);
                if let Err(e) = self.write_summary() {
                    log::error!("Failed to write dependency due to {}", e);
                }
            }
        });
    }

    pub async fn install_periodic_write_async(self: Arc<Self>) {
        let mut interval_timer = tokio::time::interval(WRITE_PERIOD);
        loop {
            interval_timer.tick().await;
            if let Err(e) = self.write_summary() {
                log::error!("Failed to write dependency due to {}", e);
            }
        }
    }

    fn record_span_succeeds_inner(&self, current: String, parent_sr: &SpanRecord) {
        log::trace!("Succeed {}", self.map_record(parent_sr));
        self.span_markov.observe(
            self.map_record(parent_sr),
            current,
            SUCCESS_STATE.to_string(),
        );
        self.fail_bernoulli.observe_absent(
            self.map_record(parent_sr),
            self.map_subrecord(parent_sr),
        );
        self.fail_bernoulli.observe_absent(
            self.map_record(parent_sr),
            TOTAL_SUBRECORD.to_string(),
        );
        // self.time_normal.observe(
        //     self.map_record(parent_sr),
        //     SUCCESS_STATE.to_string(),
        //     parent_sr.elapsed().as_secs_f64(),
        // );
    }

    fn record_span_fails_inner(&self, current: String, parent_sr: &SpanRecord) {
        log::trace!("Fail {}", self.map_record(parent_sr));
        self.span_markov.observe(
            self.map_record(parent_sr),
            current,
            FAILURE_STATE.to_string(),
        );
        self.fail_bernoulli.observe_present(
            self.map_record(parent_sr),
            self.map_subrecord(parent_sr),
        );
        self.fail_bernoulli.observe_present(
            self.map_record(parent_sr),
            TOTAL_SUBRECORD.to_string(),
        );
        // self.time_normal.observe(
        //     self.map_record(parent_sr),
        //     FAILURE_STATE.to_string(),
        //     parent_sr.elapsed().as_secs_f64(),
        // );
    }

    fn map_record(&self, sr: &SpanRecord) -> String {
        // if sr.failing {
        //     format!("{} [FAIL]", sr.name)
        // } else {
        //     sr.name.to_string()
        // }
        sr.name.to_string()
    }

    fn map_subrecord(&self, sr: &SpanRecord) -> String {
        sr.failing_subspans
            .iter()
            // .sorted()  // no need since BTreeSet
            .join(", ")
    }
}