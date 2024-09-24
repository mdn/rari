use std::{
    fmt,
    sync::{Arc, Mutex},
};
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{registry::LookupSpan, Layer};

#[derive(Debug)]
pub struct Issue {
    pub trace: Vec<(&'static str, String)>,
}

#[derive(Clone)]
pub struct InMemoryLayer {
    events: Arc<Mutex<Vec<Issue>>>,
}

impl InMemoryLayer {
    pub fn new() -> Self {
        InMemoryLayer {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> Arc<Mutex<Vec<Issue>>> {
        Arc::clone(&self.events)
    }
}

impl Visit for Issue {
    fn record_debug(&mut self, _: &Field, _: &dyn fmt::Debug) {}
    fn record_str(&mut self, field: &Field, value: &str) {
        self.trace.push((field.name(), value.to_string()));
    }
}
impl<S> Layer<S> for InMemoryLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event, _: tracing_subscriber::layer::Context<S>) {
        let mut issue = Issue { trace: vec![] };
        let mut events = self.events.lock().unwrap();
        event.record(&mut issue);
        events.push(issue);
    }
}
