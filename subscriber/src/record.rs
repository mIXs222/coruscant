use tracing::Id;


#[derive(Clone, Debug)]
pub struct SpanRecord {
    pub id: Id,
    pub name: &'static str,
    pub latest: Option<Box<SpanRecord>>,
}

impl SpanRecord {
    pub fn new(id: Id, name: &'static str) -> SpanRecord {
        SpanRecord {
          id,
          name,
          latest: None,
        }
    }
}

pub const ROOT_SPAN: &str = "__ROOT_SPAN__";
