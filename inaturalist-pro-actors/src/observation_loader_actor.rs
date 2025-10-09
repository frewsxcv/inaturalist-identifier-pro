use actix::prelude::*;
use inaturalist_pro_core::AppMessage;
use inaturalist_pro_geo::{GeohashGrid, GeohashObservations};
use tokio::sync::mpsc::UnboundedSender;

use crate::observation_processor_actor::{ObservationProcessorActor, ProcessObservationMessage};

pub struct ObservationLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub grid: GeohashGrid,
    pub api_token: String,
    pub request: inaturalist::apis::observations_api::ObservationsGetParams,
    pub soft_limit: std::sync::Arc<std::sync::atomic::AtomicI32>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartLoadingMessage;

impl Actor for ObservationLoaderActor {
    type Context = Context<Self>;
}

impl Handler<StartLoadingMessage> for ObservationLoaderActor {
    type Result = ();

    fn handle(&mut self, _msg: StartLoadingMessage, ctx: &mut Self::Context) {
        tracing::info!("Starting observation loading...");
        let grid = self.grid.clone();
        let tx_app_message = self.tx_app_message.clone();
        let api_token = self.api_token.clone();
        let request = self.request.clone();
        let soft_limit = self.soft_limit.clone();

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
