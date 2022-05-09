use tracing::Id;


#[derive(Clone, Debug)]
pub struct SpanRecord {
    pub id: Id,
    pub name: &'static str,
    pub latest: Option<Box<SpanRecord>>,
    pub failing: bool,
}

impl SpanRecord {
    pub fn new(id: Id, name: &'static str) -> SpanRecord {
        SpanRecord {
          id,
          name,
          latest: None,
          failing: false,
        }
    }
}

pub const ROOT_SPAN: &str = "__ROOT_SPAN__";
