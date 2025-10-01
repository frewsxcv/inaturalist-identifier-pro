use crate::actors::observation_processor_actor::{
    ObservationProcessorActor, ProcessObservationMessage,
};
use crate::geohash_ext::GeohashGrid;
use crate::{geohash_observations::GeohashObservations, operations::Operation};
use actix::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

pub struct ObservationLoaderActor {
    pub tx_app_message: UnboundedSender<crate::AppMessage>,
    pub grid: GeohashGrid,
    pub api_token: String,
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
                        crate::fetch_soft_limit(),
                        crate::CurOperation::request(),
                        &api_token,
                    )
                    .await
                {
                    Ok(_) => {
                        if let Err(e) = tx_app_message.clone().send(crate::AppMessage::Progress) {
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
