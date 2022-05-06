use serde::Serialize;
use std::io::Write;
use std::fs::OpenOptions;
use std::time::Duration;
use std::sync::Arc;

use crate::bernoulli::CategoryBernEstimator;
use crate::bernoulli::CategoryBernSummary;
use crate::markov::ContextMarkovEstimator;
use crate::markov::ContextMarkovSummary;
use crate::record::SpanRecord;


const INITIAL_STATE: &str = "__INITIAL_STATE__";
const WRITE_PERIOD: Duration = Duration::from_secs(10);
const DEPENDENCY_OUT: &str = "dependency_summary.jsons";

#[derive(Serialize, Debug)]
pub struct DependencySummary {
    span_markov: ContextMarkovSummary<String>,
    fail_bernoulli: CategoryBernSummary<String>,
}

impl DependencySummary {
    pub fn make_span_markov(&self) -> ContextMarkovSummary<String> {
        // TODO: make model out of summary
        self.span_markov.clone()
    }

    pub fn make_fail_bernoulli(&self) -> CategoryBernSummary<String> {
        // TODO: make model out of summary
        self.fail_bernoulli.clone()
    }
}

/* Process dependency data */
pub struct DependencyProcessor {
    span_markov: ContextMarkovEstimator<String>,
    fail_bernoulli: CategoryBernEstimator<String>,
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
            fail_bernoulli: CategoryBernEstimator::default(),
        }
    }

    pub fn record_span(&self, current_sr: &SpanRecord, parent_sr: &SpanRecord) {
        log::trace!("Span {} [ {} ]", parent_sr.name, current_sr.name);
        self.span_markov.observe(
            parent_sr.name.to_string(),
            INITIAL_STATE.to_string(),
            current_sr.name.to_string(),
        );
        self.fail_bernoulli.observe(current_sr.name.to_string());
    }

    pub fn record_span_follows(&self, current_sr: &SpanRecord, prev_sr: &SpanRecord, parent_sr: &SpanRecord) {
        log::trace!("Span {} [ {} --> {} ]", parent_sr.name, prev_sr.name, current_sr.name);
        self.span_markov.observe(
            parent_sr.name.to_string(),
            prev_sr.name.to_string(),
            current_sr.name.to_string(),
        );
        self.fail_bernoulli.observe(current_sr.name.to_string());
    }

    pub fn record_span_fails(&self, current_sr: &SpanRecord) {
        log::trace!("Fail {}", current_sr.name);
        self.fail_bernoulli.observe_event(current_sr.name.to_string());
    }

    pub fn summarize(&self) -> DependencySummary {
        DependencySummary {
            span_markov: self.span_markov.clone().summarize(),
            fail_bernoulli: self.fail_bernoulli.clone().summarize(),
        }
    }

    fn write(&self) -> std::io::Result<()> {
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
                if let Err(e) = self.write() {
                    log::error!("Failed to write dependency due to {}", e);
                }
            }
        });
    }

    pub async fn install_periodic_write_async(self: Arc<Self>) {
        let mut interval_timer = tokio::time::interval(WRITE_PERIOD);
        loop {
            interval_timer.tick().await;
            if let Err(e) = self.write() {
                log::error!("Failed to write dependency due to {}", e);
            }
        }
    }
}