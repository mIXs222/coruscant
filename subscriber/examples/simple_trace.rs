use rand::Rng;
use std::error::Error;
use tracing_subscriber::prelude::*;

use coruscant_subscriber::dependency::DependencyLayer;


type GenericError = Box<dyn Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

#[tracing::instrument]
fn call_a() -> Result<()> {
    call_b()?;
    for _ in 0 .. 4 {
        call_c()?;
    }
    call_d()
}

#[tracing::instrument]
fn call_b() -> Result<()> {
    if rand::thread_rng().gen_bool(0.5) {
        tracing::error!("B failed");
        return call_d()
    }
    Ok(())
}

#[tracing::instrument]
fn call_c() -> Result<()> {
    if rand::thread_rng().gen_bool(0.1) {
        tracing::error!("C failed");
        return Err("C randomly failed")?
    }
    Ok(())
}

#[tracing::instrument]
fn call_d() -> Result<()> {
    if rand::thread_rng().gen_bool(0.1) {
        tracing::error!("D failed");
        return Err("D randomly failed")?
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
    let subscriber = tracing_subscriber::Registry::default()
      .with(DependencyLayer::new());
      // .with(DummyLayer::new());
      // .with(ErrorLayer::default());
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting global default failed");

    // run the program
    for _ in 0..5 {
      match call_a() {
        Ok(()) => {},
        Err(e) => println!("!!! FAILED !!! {}", e),
      }
    }
}