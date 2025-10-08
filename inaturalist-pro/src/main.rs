use actix::{prelude::*, SystemRegistry};
use actors::{
    IdentifyActor, OauthActor, ObservationLoaderActor, ObservationProcessorActor, TaxaLoaderActor,
    TaxonTreeBuilderActor,
};
use geohash_ext::GeohashGrid;
use inaturalist::models::{Observation, ShowTaxon};
use inaturalist_oauth::{Authenticator, TokenDetails};
use inaturalist_pro_core::AppMessage;
use inaturalist_pro_ui::Ui;
use oauth2::AuthorizationCode;
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
mod places;
mod utils;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

type ObservationId = i32;

use std::sync::OnceLock;

static FETCH_SOFT_LIMIT_CELL: OnceLock<sync::atomic::AtomicI32> = OnceLock::new();
fn fetch_soft_limit() -> &'static sync::atomic::AtomicI32 {
    FETCH_SOFT_LIMIT_CELL.get_or_init(|| sync::atomic::AtomicI32::new(30))
}

type CurOperation = operations::TopImageScore;

#[actix::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let cfg: MyConfig = confy::load("inaturalist-identifier-pro", None)?;
    let client_id = "h_gk-W1QMcTwTAH4pmo3TEitkJzeeZphpsj7TM_yq18".to_string();
    let client_secret = "RLRDkivCGzGMGqWrV4WHIA7NJ7CqL0nhQ5n9lbIipCw".to_string();
    let _authenticator = Authenticator::new(client_id.clone(), client_secret.clone());

    // Check if we have a valid token, but don't block on authentication
    let api_token = if let Some(token) = cfg.token {
        if token.expires_at >= std::time::SystemTime::now() {
            Some(token.api_token)
        } else {
            tracing::info!("Token expired, will need to re-authenticate");
            None
        }
    } else {
        tracing::info!("No token found, user can authenticate from the UI");
        None
    };

    let grid = GeohashGrid::from_rect(places::nyc().clone(), 4);

    let operation = CurOperation::default();

    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    let observation_loader_addr =
        ObservationLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
            let tx_app_message = tx_app_message.clone();
            let api_token = api_token.clone().unwrap_or_default();
            |_ctx| ObservationLoaderActor {
                tx_app_message,
                grid,
                api_token,
            }
        });

    let addr = TaxonTreeBuilderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone().unwrap_or_default();
        move |_ctx| TaxonTreeBuilderActor {
            tx_app_message,
            api_token,
        }
    });
    SystemRegistry::set(addr);

    let addr = ObservationProcessorActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone().unwrap_or_default();
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
        let api_token = api_token.clone().unwrap_or_default();
        {
            |_ctx| IdentifyActor { api_token }
        }
    });
    SystemRegistry::set(addr);

    let addr = TaxaLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone().unwrap_or_default();
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

    let oauth_addr = OauthActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        |_ctx| OauthActor::new(tx_app_message)
    });

    eframe::run_native(
        "iNaturalist Pro",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            let mut app = crate::app::App::default();
            app.tx_app_message = tx_app_message;
            app.rx_app_message = rx_app_message;
            app.observation_loader_addr = Some(observation_loader_addr);
            app.oauth_addr = oauth_addr;
            app.state.is_authenticated = api_token.is_some();
            app.api_token = api_token;
            app.client_id = Some(client_id);
            app.client_secret = Some(client_secret);
            app.ui = Ui::new(app.tx_app_message.clone());
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}
