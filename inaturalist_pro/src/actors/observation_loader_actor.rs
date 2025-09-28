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

impl Actor for ObservationLoaderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
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
                GeohashObservations(geohash)
                    .fetch_from_api(
                        |observation| {
                            ObservationProcessorActor::from_registry()
                                .try_send(ProcessObservationMessage { observation })
                                .unwrap();
                        },
                        crate::fetch_soft_limit(),
                        crate::CurOperation::request(),
                        &api_token,
                    )
                    .await
                    .unwrap();
                tx_app_message
                    .clone()
                    .send(crate::AppMessage::Progress)
                    .unwrap();
            }
            // FIXME: call below
            // operation.lock().await.finish();
            tracing::info!("Finished loading thread");
            // tx.send(AppMessage::Results(std::mem::take(
            //     &mut operation.0,
            // )))
            // .await
            // .unwrap();
        };

        ctx.spawn(Box::pin(t).into_actor(self));
    }
}
