use actix::{prelude::*, SystemRegistry};
use actors::{
    IdentifyActor, ObservationLoaderActor, ObservationProcessorActor, TaxaLoaderActor,
    TaxonTreeBuilderActor,
};
use geohash_ext::GeohashGrid;
use inaturalist::models::{Observation, ShowTaxon};
use inaturalist_oauth::{Authenticator, TokenDetails};
use serde::{Deserialize, Serialize};
use std::{error, sync};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct MyConfig {
    token: Option<TokenDetails>,
}

mod actors;
mod app;
mod geohash_ext;
mod geohash_observations;
mod operations;
mod panels;

mod places;
mod taxa_store;
mod taxon_tree;
mod utils;
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

    let mut cfg: MyConfig = confy::load("inaturalist-identifier-pro", None)?;
    let client_id = "h_gk-W1QMcTwTAH4pmo3TEitkJzeeZphpsj7TM_yq18".to_string();
    let client_secret = "RLRDkivCGzGMGqWrV4WHIA7NJ7CqL0nhQ5n9lbIipCw".to_string();
    let authenticator = Authenticator::new(client_id, client_secret);

    let token = if let Some(token) = cfg.token {
        if token.expires_at < std::time::SystemTime::now() {
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

    let addr = ObservationProcessorActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone();
        {
            |_ctx| ObservationProcessorActor {
                tx_app_message,
                operation,
                api_token,
            }
        }
    });
    SystemRegistry::set(addr);

    let addr = IdentifyActor::start_in_arbiter(&Arbiter::new().handle(), {
        let _tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone();
        {
            |_ctx| IdentifyActor { api_token }
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
            let mut app = crate::app::App::default();
            app.tx_app_message = tx_app_message;
            app.rx_app_message = rx_app_message;
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}
