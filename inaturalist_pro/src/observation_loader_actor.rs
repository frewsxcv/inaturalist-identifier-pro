use crate::{geohash_observations::GeohashObservations, operations::Operation};
use tokio::sync::mpsc::UnboundedSender;
use actix::prelude::*;
use crate::geohash_ext::GeohashGrid;
use inaturalist::models::Observation;

pub struct ObservationLoaderActor {
    pub tx_app_message: UnboundedSender<crate::AppMessage>,
    pub grid: GeohashGrid,
    pub tx_load_observations: UnboundedSender<Observation>,
}

/// Declare actor and its context
impl Actor for ObservationLoaderActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let grid = self.grid.clone();
        let tx_load_observations = self.tx_load_observations.clone();
        let tx_app_message = self.tx_app_message.clone();
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
                        tx_load_observations.clone(),
                        &crate::FETCH_SOFT_LIMIT,
                        crate::CurOperation::request(),
                    )
                    .await
                    .unwrap();
                tx_app_message.clone().send(crate::AppMessage::Progress).unwrap();
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