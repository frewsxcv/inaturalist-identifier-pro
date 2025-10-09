use actix::prelude::*;
use inaturalist_pro_core::AppMessage;
use inaturalist_pro_geo::{GeohashGrid, GeohashObservations};
use std::collections::{HashSet, VecDeque};
use tokio::sync::mpsc::UnboundedSender;

use crate::observation_processor_actor::{ObservationProcessorActor, ProcessObservationMessage};

type TaxonId = i32;

/// Consolidated actor for all iNaturalist API requests
/// Manages rate limiting across all request types
pub struct ApiLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub api_token: String,

    // Taxa loading state
    pub taxa_to_load: HashSet<TaxonId>,
    pub taxa_loaded: HashSet<TaxonId>,

    // Request queue for batching and rate limiting
    pub pending_requests: VecDeque<ApiRequest>,

    // Flag to track if we're currently processing
    pub is_processing: bool,
}

impl Default for ApiLoaderActor {
    fn default() -> Self {
        unimplemented!("ApiLoaderActor must be created with explicit parameters")
    }
}

impl SystemService for ApiLoaderActor {}

impl Supervised for ApiLoaderActor {}

impl Actor for ApiLoaderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Set up periodic processing of queued requests
        ctx.run_interval(std::time::Duration::from_millis(500), |act, ctx| {
            if !act.is_processing && !act.pending_requests.is_empty() {
                ctx.notify(ProcessNextRequestMessage);
            }
        });
    }
}

/// Enum representing different types of API requests
#[derive(Clone)]
pub enum ApiRequest {
    LoadTaxon(TaxonId),
    FetchTaxa,
    FetchCurrentUser,
    LoadObservations {
        grid: GeohashGrid,
        request: inaturalist::apis::observations_api::ObservationsGetParams,
        soft_limit: std::sync::Arc<std::sync::atomic::AtomicI32>,
    },
}

// ============================================================================
// Messages
// ============================================================================

#[derive(Message)]
#[rtype(result = "()")]
pub struct LoadTaxonMessage(pub TaxonId);

#[derive(Message)]
#[rtype(result = "()")]
pub struct FetchTaxaMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct FetchCurrentUserMessage;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartLoadingObservationsMessage {
    pub grid: GeohashGrid,
    pub request: inaturalist::apis::observations_api::ObservationsGetParams,
    pub soft_limit: std::sync::Arc<std::sync::atomic::AtomicI32>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ProcessNextRequestMessage;

// ============================================================================
// Message Handlers
// ============================================================================

impl Handler<LoadTaxonMessage> for ApiLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: LoadTaxonMessage, ctx: &mut Self::Context) -> Self::Result {
        self.taxa_to_load.insert(msg.0);
        // Trigger taxa fetch if we have enough or after a delay
        ctx.notify_later(FetchTaxaMessage, std::time::Duration::from_millis(100));
    }
}

impl Handler<FetchTaxaMessage> for ApiLoaderActor {
    type Result = ();

    fn handle(&mut self, _msg: FetchTaxaMessage, ctx: &mut Self::Context) -> Self::Result {
        let taxa_ids_to_fetch: Vec<TaxonId> = self
            .taxa_to_load
            .difference(&self.taxa_loaded)
            .copied()
            .take(30) // Maximum number allowed by API
            .collect();

        if taxa_ids_to_fetch.is_empty() {
            return;
        }

        let tx_app_message = self.tx_app_message.clone();
        let api_token = self.api_token.clone();

        // Mark as loaded immediately to prevent duplicate requests
        for &taxon_id in &taxa_ids_to_fetch {
            self.taxa_loaded.insert(taxon_id);
        }

        ctx.wait(
            Box::pin(async move {
                match inaturalist_fetch::fetch_taxa(taxa_ids_to_fetch.clone(), &api_token).await {
                    Ok(taxa) => {
                        for taxon in taxa.results {
                            let _ = tx_app_message.send(AppMessage::TaxonLoaded(Box::new(taxon)));
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch taxa: {:?}", e);
                    }
                }
            })
            .into_actor(self),
        );
    }
}

impl Handler<FetchCurrentUserMessage> for ApiLoaderActor {
    type Result = ();

    fn handle(&mut self, _msg: FetchCurrentUserMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();
        let api_token = self.api_token.clone();

        ctx.wait(
            Box::pin(async move {
                match inaturalist_fetch::fetch_current_user(&api_token).await {
                    Ok(user) => {
                        let _ = tx_app_message.send(AppMessage::UserLoaded(user));
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch current user: {:?}", e);
                    }
                }
            })
            .into_actor(self),
        );
    }
}

impl Handler<StartLoadingObservationsMessage> for ApiLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: StartLoadingObservationsMessage, ctx: &mut Self::Context) {
        tracing::info!("Starting observation loading...");
        let grid = msg.grid;
        let tx_app_message = self.tx_app_message.clone();
        let api_token = self.api_token.clone();
        let request = msg.request;
        let soft_limit = msg.soft_limit;

        let t = async move {
            for (i, geohash) in grid.clone().0.into_iter().enumerate() {
                tracing::info!(
                    "Fetch observations for geohash {} ({} / {})",
                    geohash.string,
                    i + 1,
                    grid.0.len(),
                );

                match GeohashObservations(geohash)
                    .fetch_from_api(
                        |observation| {
                            if let Err(e) = ObservationProcessorActor::from_registry()
                                .try_send(ProcessObservationMessage { observation })
                            {
                                tracing::warn!("Failed to send observation to processor: {}", e);
                            }
                        },
                        &soft_limit,
                        request.clone(),
                        &api_token,
                    )
                    .await
                {
                    Ok(_) => {
                        if let Err(e) = tx_app_message.clone().send(AppMessage::Progress) {
                            tracing::error!("Failed to send progress message: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch observations for geohash: {}", e);
                        // Continue to next geohash instead of crashing
                        continue;
                    }
                }
            }

            tracing::info!("Finished loading observations");
        };

        ctx.spawn(Box::pin(t).into_actor(self));
    }
}

impl Handler<ProcessNextRequestMessage> for ApiLoaderActor {
    type Result = ();

    fn handle(
        &mut self,
        _msg: ProcessNextRequestMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        // For future enhancement: implement request queue processing
        // This would allow more sophisticated rate limiting and prioritization
        self.is_processing = false;
    }
}
