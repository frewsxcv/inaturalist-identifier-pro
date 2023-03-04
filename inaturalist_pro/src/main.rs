#![feature(async_closure)]

use actix::{prelude::*, SystemRegistry};
use geohash_ext::GeohashGrid;
use image_store_actor::ImageStoreActor;
use inaturalist::models::Observation;
use observation_loader_actor::ObservationLoaderActor;
use observation_processor_actor::ObservationProcessorActor;
use std::{error, sync};

mod app;
mod geohash_ext;
mod geohash_observations;
mod image_store;
mod image_store_actor;
mod observation_loader_actor;
mod observation_processor_actor;
mod operations;
mod places;
mod taxon_tree;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

#[derive(Debug)]
pub enum AppMessage {
    Progress,
    Result(
        (
            Box<Observation>,
            Vec<inaturalist_fetch::ComputerVisionObservationScore>,
        ),
    ),
}

lazy_static::lazy_static! {
    static ref FETCH_SOFT_LIMIT: sync::atomic::AtomicI32 = sync::atomic::AtomicI32::new(5000);
}

type CurOperation = operations::TopImageScore;

#[actix::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*places::NYC, 4);

    let operation = CurOperation::default();

    let (tx_load_observations, mut rx_load_observations) =
        tokio::sync::mpsc::unbounded_channel::<Observation>();
    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    ObservationLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let grid = grid.clone();
        |_ctx| ObservationLoaderActor {
            tx_app_message,
            tx_load_observations,
            grid,
        }
    });

    let image_store = sync::Arc::new(sync::RwLock::new(image_store::ImageStore::default()));

    let addr = ImageStoreActor::start_in_arbiter(&Arbiter::new().handle(), {
        let image_store = image_store.clone();
        |_| ImageStoreActor { image_store }
    });
    SystemRegistry::set(addr);

    let total_geohashes = grid.0.len();

    ObservationProcessorActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        |_ctx| observation_processor_actor::ObservationProcessorActor {
            tx_app_message,
            operation,
        }
    });

    eframe::run_native(
        "eframe template",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Box::new(crate::app::TemplateApp {
                rx_app_message,
                loaded_geohashes: 0,
                total_geohashes,
                results: vec![],
                image_store,
            })
        }),
    )?;

    Ok(())
}
