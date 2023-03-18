#![feature(async_closure)]

use actix::{prelude::*, SystemRegistry};
use geohash_ext::GeohashGrid;
use image_store_actor::ImageStoreActor;
use inaturalist::models::Observation;
use observation_loader_actor::ObservationLoaderActor;
use observation_processor_actor::ObservationProcessorActor;
use std::{collections, error, sync};
use taxon_tree_builder_actor::TaxonTreeBuilderActor;

mod app;
mod geohash_ext;
mod geohash_observations;
mod image_store;
mod image_store_actor;
mod observation_loader_actor;
mod observation_processor_actor;
mod operations;
mod places;
mod taxa_store;
mod taxon_tree;
mod taxon_tree_builder_actor;

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
    TaxonTree {
        observation_id: i32,
        taxon_tree: taxon_tree::TaxonTree,
    },
}

lazy_static::lazy_static! {
    static ref FETCH_SOFT_LIMIT: sync::atomic::AtomicI32 = sync::atomic::AtomicI32::new(30);
}

type CurOperation = operations::TopImageScore;

#[actix::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*places::NYC, 4);

    let operation = CurOperation::default();

    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    ObservationLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        |_ctx| ObservationLoaderActor {
            tx_app_message,
            grid,
        }
    });

    let addr = TaxonTreeBuilderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        move |_ctx| TaxonTreeBuilderActor { tx_app_message }
    });
    SystemRegistry::set(addr);

    let image_store = sync::Arc::new(sync::RwLock::new(image_store::ImageStore::default()));
    let addr = ImageStoreActor::start_in_arbiter(&Arbiter::new().handle(), {
        let image_store = image_store.clone();
        |_| ImageStoreActor { image_store }
    });
    SystemRegistry::set(addr);

    let addr = ObservationProcessorActor::start_in_arbiter(&Arbiter::new().handle(), {
        |_ctx| observation_processor_actor::ObservationProcessorActor {
            tx_app_message,
            operation,
        }
    });
    SystemRegistry::set(addr);

    eframe::run_native(
        "eframe template",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Box::new(crate::app::App {
                rx_app_message,
                loaded_geohashes: 0,
                results: vec![],
                image_store,
                taxa_store: crate::taxa_store::TaxaStore(collections::HashMap::new()),
            })
        }),
    )?;

    Ok(())
}
