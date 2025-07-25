use actix::{prelude::*, SystemRegistry};
use geohash_ext::GeohashGrid;
use identify_actor::IdentifyActor;
use image_store_actor::ImageStoreActor;
use inaturalist::models::{Observation, ShowTaxon};
use observation_loader_actor::ObservationLoaderActor;
use observation_processor_actor::ObservationProcessorActor;
use std::{error, sync};
use taxa_loader_actor::TaxaLoaderActor;
use taxon_tree_builder_actor::TaxonTreeBuilderActor;

mod app;
mod geohash_ext;
mod geohash_observations;
mod identify_actor;
mod image_store;
mod image_store_actor;
mod observation_loader_actor;
mod observation_processor_actor;
mod operations;
mod places;
mod taxa_loader_actor;
mod taxa_store;
mod taxon_tree;
mod taxon_tree_builder_actor;
mod widgets;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

type ObservationId = i32;

#[derive(Debug)]
pub enum AppMessage {
    Progress,
    TaxonLoaded(Box<ShowTaxon>),
    SkipCurrentObservation,
    ObservationLoaded(Box<Observation>),
    ComputerVisionScoreLoaded(ObservationId, Vec<inaturalist_fetch::ComputerVisionObservationScore>),
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
        let tx_app_message = tx_app_message.clone();
        {
            |_ctx| observation_processor_actor::ObservationProcessorActor {
                tx_app_message,
                operation,
            }
        }
    });
    SystemRegistry::set(addr);

    let addr = IdentifyActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        {
            |_ctx| IdentifyActor { tx_app_message }
        }
    });
    SystemRegistry::set(addr);

    let addr = TaxaLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        {
            |_ctx| TaxaLoaderActor {
                tx_app_message,
                loaded: Default::default(),
                to_load: Default::default(),
            }
        }
    });
    SystemRegistry::set(addr);

    eframe::run_native(
        "eframe template",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Box::new(crate::app::App {
                tx_app_message,
                rx_app_message,
                loaded_geohashes: 0,
                results: vec![],
                image_store,
                taxa_store: Default::default(),
                current_observation_id: None,
            })
        }),
    )?;

    Ok(())
}
