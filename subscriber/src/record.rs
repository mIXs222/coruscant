use std::collections::BTreeSet;
use std::time::Duration;
use std::time::Instant;
use tracing::Id;


#[derive(Clone, Debug)]
pub struct SpanRecord {
    pub id: Id,
    pub name: &'static str,
    pub creation_time: Instant,
    pub latest: Option<Box<SpanRecord>>,
    pub failing: bool,
    pub failing_subspans: BTreeSet<String>,
}

impl SpanRecord {
    pub fn new(id: Id, name: &'static str) -> SpanRecord {
        SpanRecord {
          id,
          name,
          creation_time: Instant::now(),
          latest: None,
          failing: false,
          failing_subspans: BTreeSet::new(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.creation_time.elapsed()
    }
}

pub const ROOT_SPAN: &str = "__ROOT_SPAN__";
