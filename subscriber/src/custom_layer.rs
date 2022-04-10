use std::collections::BTreeMap;
use tracing_subscriber::Layer;
use tracing::{
    Id, Metadata, Event,
    span,
    subscriber::{self, Subscriber},
};

#[derive(Debug)]
struct CustomFieldStorage(BTreeMap<String, serde_json::Value>);

pub struct CustomLayer;

impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    )  {
        let span = ctx.span(id).unwrap();
        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        attrs.record(&mut visitor);
        let storage = CustomFieldStorage(fields);
        let mut extensions = span.extensions_mut();
        extensions.insert(storage);
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        let mut extensions_mut = span.extensions_mut();
        let custom_field_storage: &mut CustomFieldStorage = extensions_mut.get_mut::<CustomFieldStorage>().unwrap();
        let json_data: &mut BTreeMap<String, serde_json::Value> = &mut custom_field_storage.0;
        let mut visitor = JsonVisitor(json_data);
        values.record(&mut visitor);
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let scope = ctx.event_scope(event).unwrap();
        let mut spans = vec![];
        for span in scope.from_root() {
            let extensions = span.extensions();
            let storage = extensions.get::<CustomFieldStorage>().unwrap();
            let field_data: &BTreeMap<String, serde_json::Value> = &storage.0;
            spans.push(serde_json::json!({
                "target": span.metadata().target(),
                "name": span.name(),
                "level": format!("{:?}", span.metadata().level()),
                "fields": field_data,
            }));
        }

        let mut fields = BTreeMap::new();
        let mut visitor = JsonVisitor(&mut fields);
        event.record(&mut visitor);

        let output = serde_json::json!({
            "target": event.metadata().target(),
            "name": event.metadata().name(),
            "level": format!("{:?}", event.metadata().level()),
            "fields": fields,
            "spans": spans,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}

struct JsonVisitor<'a>(&'a mut BTreeMap<String, serde_json::Value>);

impl<'a> tracing::field::Visit for JsonVisitor<'a> {

    fn record_follows_from(&mut self, field: &tracing::field::Field, value: &tracing::span::Id) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(value.as_u64()),
        );
    }
    
    fn record_dependency(&mut self, field: &tracing::field::Field, parent: &tracing::span::Id, child: &tracing::span::Id) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(parent.as_u64()),
            serde_json::json!(child.as_u64()),
        );
    }
    
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(value.to_string()),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }
    
}
