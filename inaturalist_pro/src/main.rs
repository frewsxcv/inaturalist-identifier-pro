use actix::{prelude::*, SystemRegistry};
use geohash_ext::GeohashGrid;
use identify_actor::IdentifyActor;
use image_store_actor::ImageStoreActor;
use inaturalist::models::{Observation, ShowTaxon};
use inaturalist_oauth::{Authenticator, TokenDetails};
use observation_loader_actor::ObservationLoaderActor;
use observation_processor_actor::ObservationProcessorActor;
use serde::{Deserialize, Serialize};
use std::{error, sync};
use taxa_loader_actor::TaxaLoaderActor;
use taxon_tree_builder_actor::TaxonTreeBuilderActor;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct MyConfig {
    token: Option<TokenDetails>,
}

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
    ComputerVisionScoreLoaded(
        ObservationId,
        Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ),
    TaxonTree {
        observation_id: i32,
        taxon_tree: taxon_tree::TaxonTree,
    },
}

use std::sync::OnceLock;

static FETCH_SOFT_LIMIT_CELL: OnceLock<sync::atomic::AtomicI32> = OnceLock::new();
fn fetch_soft_limit() -> &'static sync::atomic::AtomicI32 {
    FETCH_SOFT_LIMIT_CELL.get_or_init(|| sync::atomic::AtomicI32::new(30))
}

type CurOperation = operations::TopImageScore;

#[actix::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let mut cfg: MyConfig = confy::load("inaturalist-fetch", None)?;
    let client_id = "h_gk-W1QMcTwTAH4pmo3TEitkJzeeZphpsj7TM_yq18".to_string();
    let client_secret = "RLRDkivCGzGMGqWrV4WHIA7NJ7CqL0nhQ5n9lbIipCw".to_string();
    let authenticator = Authenticator::new(client_id, client_secret);

    let token = if let Some(token) = cfg.token {
        if token.expires_at < chrono::Utc::now() {
            let new_token = authenticator.get_api_token().await?;
            cfg.token = Some(new_token.clone());
            confy::store("inaturalist-fetch", None, cfg.clone())?;
            new_token
        } else {
            token
        }
    } else {
        let token = authenticator.get_api_token().await?;
        cfg.token = Some(token.clone());
        confy::store("inaturalist-fetch", None, cfg.clone())?;
        token
    };
    let api_token = token.api_token;

    let grid = GeohashGrid::from_rect(places::nyc().clone(), 4);

    let operation = CurOperation::default();

    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    ObservationLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone();
        |_ctx| ObservationLoaderActor {
            tx_app_message,
            grid,
            api_token,
        }
    });

    let addr = TaxonTreeBuilderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone();
        move |_ctx| TaxonTreeBuilderActor {
            tx_app_message,
            api_token,
        }
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
        let api_token = api_token.clone();
        {
            |_ctx| observation_processor_actor::ObservationProcessorActor {
                tx_app_message,
                operation,
                api_token,
            }
        }
    });
    SystemRegistry::set(addr);

    let addr = IdentifyActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone();
        {
            |_ctx| IdentifyActor {
                tx_app_message,
                api_token,
            }
        }
    });
    SystemRegistry::set(addr);

    let addr = TaxaLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone();
        {
            |_ctx| TaxaLoaderActor {
                tx_app_message,
                loaded: Default::default(),
                to_load: Default::default(),
                api_token,
            }
        }
    });
    SystemRegistry::set(addr);

    eframe::run_native(
        "iNaturalist Identifier Pro",
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
