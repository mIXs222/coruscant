use chashmap::CHashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::Event;
use tracing::Id;
use tracing::span;
use tracing::Level;
use tracing::subscriber::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use tracing_subscriber::registry::LookupSpan;

use crate::processor::DependencyProcessor;
use crate::record::ROOT_SPAN;
use crate::record::SpanRecord;

/// A subscriber layer looking for reliability dependency
pub struct DependencyLayer {
  records: CHashMap<Id, SpanRecord>,
  root_sr: RwLock<SpanRecord>,
  processor: Arc<DependencyProcessor>,
}

impl DependencyLayer {
    pub fn construct() -> (DependencyLayer, Arc<DependencyProcessor>) {
        let processor = Arc::new(DependencyProcessor::new());
        let layer = DependencyLayer {
            records: CHashMap::new(),
            root_sr: RwLock::new(SpanRecord::new(Id::from_u64(1), ROOT_SPAN)),
            processor: processor.clone(),
        };
        (layer, processor)
    }

    fn stacked_span_id(&self, current_sr: &SpanRecord, parent_id: &Id) {
        match self.records.get_mut(parent_id) {
            Some(mut parent_sr) => self.stacked_span(current_sr, &mut parent_sr),
            None => log::warn!("Parent span record not found {:?}", parent_id),
        }
    }

    fn stacked_span(&self, current_sr: &SpanRecord, parent_sr: &mut SpanRecord) {
        if let Some(prev_sr) = &parent_sr.latest {
            self.record_follows_under(current_sr, prev_sr, parent_sr);
        } else {
            self.record_stacked_span(current_sr, parent_sr);
        }
        parent_sr.latest = Some(Box::new(current_sr.clone()));
    }

    fn rooted_span(&self, current_sr: &SpanRecord) {
        if let Some(prev_sr) = &self.root_sr.read().unwrap().latest {
            self.record_follows_rooted(current_sr, prev_sr);
        } else {
            self.record_rooted_span(current_sr);
        }
        self.root_sr.write().unwrap().latest = Some(Box::new(current_sr.clone()));
    }

    fn record_rooted_span(&self, current_sr: &SpanRecord) {
        self.processor.record_span(current_sr, &self.root_sr.read().unwrap());
    }

    fn record_follows_rooted(&self, current_sr: &SpanRecord, prev_sr: &SpanRecord) {
        self.processor.record_span_follows(
            current_sr,
            prev_sr,
            &self.root_sr.read().unwrap(),
        );
    }

    fn record_stacked_span(&self, current_sr: &SpanRecord, parent_sr: &SpanRecord) {
        self.processor.record_span(current_sr, parent_sr);
    }

    fn record_follows_under(&self, current_sr: &SpanRecord, prev_sr: &SpanRecord, parent_sr: &SpanRecord) {
        self.processor.record_span_follows(
            current_sr,
            prev_sr,
            parent_sr,
        );
    }

    fn record_close(&self, current_sr: &SpanRecord, parent_id: Option<&Id>) {
        match parent_id {
            Some(parent_id) => match self.records.get_mut(parent_id) {
                Some(mut parent_sr) => self.record_close_under(current_sr, &mut parent_sr),
                None => log::warn!("Parent span record not found {:?}", parent_id),
            },
            None => self.record_close_under(current_sr, &mut self.root_sr.write().unwrap())
        }
    }

    fn record_close_under(&self, current_sr: &SpanRecord, parent_sr: &mut SpanRecord) {
    // fn record_close_under(&self, current_sr: &SpanRecord) {
        if current_sr.failing {
            if let Some(latest_sr) = &current_sr.latest {
                self.processor.record_span_fails_from(latest_sr, current_sr);
                parent_sr.failing_subspans.insert(current_sr.name.to_string());
            } else {
                self.processor.record_span_fails(current_sr);
                parent_sr.failing_subspans.insert(current_sr.name.to_string());
            }
        } else if let Some(latest_sr) = &current_sr.latest {
            self.processor.record_span_succeeds_from(latest_sr, current_sr);
            // parent_sr.failing_subspans.insert(current_sr.name.to_string());
        } else {
            self.processor.record_span_succeeds(current_sr);
            // parent_sr.failing_subspans.insert(current_sr.name.to_string());
        }
    }

    // fn record_close_under(&self, current_sr: &SpanRecord) {
    //     if let Some(parent_sr) = self.records.get(parent_id) {
    //         if current_sr.failing {
    //             self.processor.record_span_fails(current_sr, &parent_sr)
    //         } else {
    //             self.processor.record_span_succeeds(current_sr, &parent_sr);
    //         }
    //     } else {
    //         log::warn!("Report close under unseen parent {:?}", parent_id);
    //         self.record_close(current_sr);
    //     }
    // }

    fn record_found_failure(&self, current_id: &Id) {
        if let Some(mut current_sr) = self.records.get_mut(current_id) {
            current_sr.failing = true;
        } else {
            log::warn!("Report failure on unseen span {:?}", current_id);
        }
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
            } else {
                self.rooted_span(&span_record)
            },
        }

        // prepare to track current span
        if let Some(maybe_sr) = self.records.insert(id.clone(), span_record) {
            log::warn!("Latest span of {:?} still remained {:?}", id, maybe_sr);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if let Some(sr) = self.records.remove(&id) {
            self.record_close(&sr, ctx.current_span().id());
            // self.record_close_under(&sr);
        } else {
            log::warn!("Closing unseen span {:?} with a record", id);
        }
    }

    // TODO: on_enter + on_exit ?

    fn on_follows_from(&self, span: &span::Id, follows: &span::Id, ctx: Context<'_, S>) {
        if let Some(span_sr) = self.records.get(span) {
            if let Some(follows_sr) = self.records.get(follows) {
                match ctx.current_span().id() {
                    Some(parent_id) => match self.records.get(parent_id) {
                        Some(parent_sr) => self.record_follows_under(&span_sr, &follows_sr, &parent_sr),
                        None => self.record_follows_rooted(&span_sr, &follows_sr),
                    },
                    None => self.record_follows_rooted(&span_sr, &follows_sr),
                };   
            }
        }
    }

    // TODO: on_id_change ?

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        if event.metadata().level() == &Level::ERROR {
            match event.parent() {
                Some(span_id) => self.record_found_failure(span_id),
                None => if let Some(span_id) = ctx.current_span().id() {
                    self.record_found_failure(span_id)
                },
            }
        }
    }

}