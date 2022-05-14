use rand::Rng;
use std::error::Error;
use tracing_subscriber::prelude::*;

use coruscant_subscriber::dependency::DependencyLayer;


type GenericError = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, GenericError>;

#[derive(Default, Debug)]
struct DefaultError {
    why: String,
}
impl DefaultError {
    fn new(why: &str) -> GenericError {
        Box::new(DefaultError { why: why.to_string() })
    }
}
impl std::fmt::Display for DefaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.why)
    }
}
impl Error for DefaultError {}
unsafe impl Send for DefaultError {}
unsafe impl Sync for DefaultError {}

const SCALE: f64 = 1.0;

#[tracing::instrument(err(Debug))]
fn call_a() -> Result<()> {
    call_b()?;
    let _e = call_e();
    for _ in 0 .. 4 {
        call_c()?;
    }
    call_d()
}

#[tracing::instrument(err(Debug))]
fn call_b() -> Result<()> {
    if rand::thread_rng().gen_bool(0.1 * SCALE) {
        // tracing::error!("B failed");
        return call_d()
    }
    Ok(())
}

#[tracing::instrument(err(Debug))]
fn call_c() -> Result<()> {
    if rand::thread_rng().gen_bool(0.1 * SCALE) {
        // tracing::error!("C failed");
        return Err(DefaultError::new("C randomly failed"))
    }
    Ok(())
}

#[tracing::instrument(err(Debug))]
fn call_d() -> Result<()> {
    if rand::thread_rng().gen_bool(0.1 * SCALE) {
        // tracing::error!("D failed");
        return Err(DefaultError::new("D randomly failed"))
    }
    Ok(())
}

#[tracing::instrument(err(Debug))]
fn call_e() -> Result<()> {
    if rand::thread_rng().gen_bool(0.1 * SCALE) {
        // tracing::error!("D failed");
        return Err(DefaultError::new("E randomly failed"))
    }
    Ok(())
}

fn main() {
    // execution init
    env_logger::Builder::from_default_env()
        .format_timestamp_micros()
        .init();

    // tested subscriber
    // let subscriber = DummySubscriber::new();
    let (dep_layer, dep_processor) = DependencyLayer::construct();
    let subscriber = tracing_subscriber::Registry::default()
      .with(dep_layer);
      // .with(DummyLayer::new());
      // .with(ErrorLayer::default());
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting global default failed");

    // print periodically
    dep_processor.clone().install_periodic_write_threaded();

    // run the program
    // loop {
    for _ in 0 .. 100 {
    // for _ in 0 .. 1_000 {
    // for _ in 0 .. 100_000 {
    // for _ in 0 .. 1_000_000 {
      match call_a() {
        Ok(()) => {},
        Err(e) => log::trace!("!!! FAILED !!! {}", e),
      }
    }

    println!("{:#?}", dep_processor.summarize());
    if let Err(e) = dep_processor.write_summary() {
        log::error!("Failed to write dependency due to {}", e);
    }
}
