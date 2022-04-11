use chashmap::CHashMap;
use tracing::Id;
use tracing::span;
use tracing::subscriber::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use tracing_subscriber::registry::LookupSpan;


#[derive(Clone, Debug)]
struct SpanRecord {
    id: Id,
    name: &'static str,
    latest: Option<Box<SpanRecord>>,
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


/// A subscriber layer looking for reliability dependency
#[derive(Default)]
pub struct DependencyLayer {
  records: CHashMap<Id, SpanRecord>,
}

impl DependencyLayer {
    pub fn new() -> DependencyLayer {
        DependencyLayer {
          records: CHashMap::new(),
        }
    }

    fn stacked_span_id(&self, current_sr: &SpanRecord, parent_id: &Id) {
        match self.records.get_mut(parent_id) {
            Some(mut parent_sr) => self.stacked_span(current_sr, &mut parent_sr),
            None => log::warn!("Latest span of {:?} was not initialized", parent_id),
        }
    }

    fn stacked_span(&self, current_sr: &SpanRecord, parent_sr: &mut SpanRecord) {
        if let Some(prev_sr) = &parent_sr.latest {
            self.follows_under(current_sr, prev_sr, Some(parent_sr));
        }
        parent_sr.latest = Some(Box::new(current_sr.clone()));
        println!("Span {:?} [ {:?} ]", parent_sr.name, current_sr.name);
    }

    fn follows_under(&self, current_sr: &SpanRecord, prev_sr: &SpanRecord, parent_sr: Option<&SpanRecord>) {
        let parent_str = format!(
            "{:?}, {:?}",
            parent_sr.map(|sr| sr.name),
            parent_sr.map(|sr| sr.id.clone())
        );
        println!("Span {} [ {:?}  -->  {:?} ]", parent_str, prev_sr.name, current_sr.name);
    }
}

impl<S> Layer<S> for DependencyLayer
where
    S: Subscriber + std::fmt::Debug + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span_record = SpanRecord::new(id.clone(), attrs.metadata().name());

        // relate to parent span
        match attrs.parent() {
            Some(parent_id) => self.stacked_span_id(&span_record, parent_id),
            None => if let Some(parent_id) = ctx.current_span().id() {
                self.stacked_span_id(&span_record, parent_id)
            },
        }

        // prepare to track current span
        if let Some(maybe_sr) = self.records.insert(id.clone(), span_record) {
            log::warn!("Latest span of {:?} still remained {:?}", id, maybe_sr);
        }
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        if self.records.remove(&id).is_none() {
            log::warn!("Closing span {:?} with a record", id);
        }
    }

    // TODO: on_enter + on_exit ?

    fn on_follows_from(&self, span: &span::Id, follows: &span::Id, ctx: Context<'_, S>) {
        if let Some(span_sr) = self.records.get(span) {
            if let Some(follows_sr) = self.records.get(follows) {
                match ctx.current_span().id() {
                    Some(parent_id) => match self.records.get(parent_id) {
                        Some(parent_sr) => self.follows_under(&span_sr, &follows_sr, Some(&parent_sr)),
                        None => self.follows_under(&span_sr, &follows_sr, None),
                    },
                    None => self.follows_under(&span_sr, &follows_sr, None),
                };   
            }
        }
    }

    // TODO: on_id_change ?
}