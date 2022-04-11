use std::sync::atomic::{Ordering, AtomicUsize};
use tracing::Event;
use tracing::Id;
use tracing::Metadata;
use tracing::span;
use tracing::subscriber::Subscriber;
use tracing::subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use chashmap::CHashMap;


/// A subscriber looking for reliability dependency
pub struct DummySubscriber {
    next_id: AtomicUsize,  // local private id
    idents: CHashMap<&'static str, Id>,  // translate from span name to private id
}

impl Default for DummySubscriber {
    fn default() -> Self {
        Self::new()
    }
}

impl DummySubscriber {
    pub fn new() -> DummySubscriber {
        DummySubscriber {
            next_id: AtomicUsize::new(1),
            idents: CHashMap::new(),
        }
    }

    fn next_id(&self) -> Id {
        Id::from_u64(self.next_id.fetch_add(1, Ordering::SeqCst) as u64)
    }
}

impl Subscriber for DummySubscriber {
    fn register_callsite(&self, meta: &Metadata<'_>) -> subscriber::Interest {
        log::debug!("register_callsite({:?})", meta);
        subscriber::Interest::always()
    }

    fn new_span(&self, new_span: &span::Attributes<'_>) -> Id {
        log::debug!("new_span({:?})", new_span);
        let name = new_span.metadata().name();
        self.idents.upsert(name, || self.next_id(), |_| ());

        (*self.idents.get(name).unwrap()).clone()
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        // ignored
        log::debug!("record({:?}, {:?})", span, follows);
    }

    fn record(&self, span: &Id, values: &span::Record<'_>) {
        // ignored
        log::debug!("record({:?}, {:?})", span, values);
    }

    fn event(&self, event: &Event<'_>) {
        // trace event
        log::debug!("event({:?})", event);
    }

    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        log::debug!("enabled({:?})", metadata);
        true
    }

    fn enter(&self, span: &Id) {
        // enter span
        log::debug!("enter({:?})", span);
    }

    fn exit(&self, span: &Id) {
        // exit span
        log::debug!("exit({:?})", span);
    }
}


/// A subscriber layer looking for reliability dependency
#[derive(Default)]
pub struct DummyLayer;

impl DummyLayer {
    pub fn new() -> DummyLayer {
        DummyLayer
    }
}

impl<S: Subscriber + std::fmt::Debug> Layer<S> for DummyLayer {
    fn on_layer(&mut self, subscriber: &mut S) {
        log::debug!("on_layer(\n\tsubscriber: {:?})", subscriber);
    }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let _ = ctx;
        log::debug!("on_new_span(\n\tattrs: {:?},\n\tid: {:?})", attrs, id);
        // log::debug!("with ctx: {:?}", ctx);
        // log::debug!("attr parent span: {:?}", attrs.parent());
        log::debug!("current span: {:?}", ctx.current_span());
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        let _ = ctx;
        log::debug!("on_enter(\n\tid: {:?})", id);
        // log::debug!("with ctx: {:?}", ctx);
    }

    // fn on_close(&self, id: Id, ctx: Context<'_, S>) {
    //     log::debug!("on_close(\n\tid: {:?})", id);
    //     log::debug!("with ctx: {:?}", ctx);
    //     let _ = ctx;
    // }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        let _ = ctx;
        log::debug!("on_exit(\n\tid: {:?})", id);
        // log::debug!("with ctx: {:?}", ctx);
    }

    fn on_follows_from(&self, span: &span::Id, follows: &span::Id, ctx: Context<'_, S>) {
        let _ = ctx;
        log::debug!("on_id_change(\n\tspan: {:?},\n\tfollows: {:?})", span, follows);
        // log::debug!("with ctx: {:?}", ctx);
    }

    fn on_id_change(&self, old: &Id, new: &Id, ctx: Context<'_, S>) {
        let _ = ctx;
        log::debug!("on_id_change(\n\told: {:?},\n\tnew: {:?})", old, new);
        // log::debug!("with ctx: {:?}", ctx);
    }
}