use actix::{prelude::*, SystemRegistry};
use inaturalist_oauth::Authenticator;
use inaturalist_pro_actors::{
    ApiFetchCurrentUserMessage, ApiLoaderActor, IdentifyActor, OauthActor,
    ObservationProcessorActor, TaxonTreeBuilderActor,
};
use inaturalist_pro_config::Config;
use inaturalist_pro_core::AppMessage;
use inaturalist_pro_geo::{places, GeohashGrid};
use inaturalist_pro_ui::Ui;
use std::{error, sync};

mod app;
mod operations;

use operations::Operation;

type ObservationId = i32;

use std::sync::OnceLock;

static FETCH_SOFT_LIMIT_CELL: OnceLock<sync::Arc<sync::atomic::AtomicI32>> = OnceLock::new();
fn fetch_soft_limit() -> &'static sync::Arc<sync::atomic::AtomicI32> {
    FETCH_SOFT_LIMIT_CELL.get_or_init(|| sync::Arc::new(sync::atomic::AtomicI32::new(30)))
}

type CurOperation = operations::TopImageScore;

#[actix::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting iNaturalist Pro...");

    let cfg = Config::load()?;
    let client_id = "h_gk-W1QMcTwTAH4pmo3TEitkJzeeZphpsj7TM_yq18".to_string();
    let client_secret = "RLRDkivCGzGMGqWrV4WHIA7NJ7CqL0nhQ5n9lbIipCw".to_string();

    // Get the API token if valid, otherwise None
    let api_token = cfg.get_api_token();

    if api_token.is_none() {
        tracing::info!("No valid token found, user can authenticate from the UI");
    }

    let grid = GeohashGrid::from_rect(places::nyc().clone(), 4);

    let operation = CurOperation::default();

    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    // Store these for later use with ApiLoaderActor
    let observation_grid = grid;
    let observation_request = CurOperation::request();
    let observation_soft_limit = fetch_soft_limit().clone();

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

    // Consolidated API loader actor
    let api_loader_addr = ApiLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let api_token = api_token.clone().unwrap_or_default();
        |_ctx| ApiLoaderActor {
            tx_app_message,
            api_token,
            taxa_to_load: Default::default(),
            taxa_loaded: Default::default(),
            pending_requests: Default::default(),
            is_processing: false,
        }
    });
    SystemRegistry::set(api_loader_addr.clone());

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
            app.api_loader_addr = Some(api_loader_addr.clone());
            app.observation_grid = Some(observation_grid);
            app.observation_request = Some(observation_request);
            app.observation_soft_limit = Some(observation_soft_limit);
            app.oauth_addr = oauth_addr;
            app.state.is_authenticated = api_token.is_some();

            // Fetch user info if already authenticated
            if api_token.is_some() {
                tracing::info!("User is already authenticated, fetching user info...");
                api_loader_addr.do_send(ApiFetchCurrentUserMessage);
            } else {
                tracing::info!("User is not authenticated");
            }

            app.api_token = api_token;
            app.client_id = Some(client_id);
            app.client_secret = Some(client_secret);
            app.ui = Ui::new(app.tx_app_message.clone());
            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}
