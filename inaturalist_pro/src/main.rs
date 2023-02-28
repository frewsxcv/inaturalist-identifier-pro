#![feature(async_fn_in_trait)]
#![feature(async_closure)]

use actix::{prelude::*, SystemRegistry};
use geohash_ext::GeohashGrid;
use geohash_observations::GeohashObservations;
use image_store_actor::ImageStoreActor;
use inaturalist::models::Observation;
use operations::Operation;
use std::{error, sync};
use tokio::sync::mpsc::UnboundedSender;

mod app;
mod geohash_ext;
mod geohash_observations;
mod image_store;
mod image_store_actor;
mod operations;
mod places;
mod taxon_tree;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

#[derive(Debug)]
pub enum AppMessage {
    Progress,
    Result(
        (
            Box<Observation>,
            Vec<inaturalist_fetch::ComputerVisionObservationScore>,
        ),
    ),
}

lazy_static::lazy_static! {
    static ref FETCH_SOFT_LIMIT: sync::atomic::AtomicI32 = sync::atomic::AtomicI32::new(5000);
}

type CurOperation = operations::TopImageScore;

struct ObservationLoaderActor {
    tx_app_message: UnboundedSender<AppMessage>,
    grid: GeohashGrid,
    tx_load_observations: UnboundedSender<Observation>,
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
                        &FETCH_SOFT_LIMIT,
                        CurOperation::request(),
                    )
                    .await
                    .unwrap();
                tx_app_message.clone().send(AppMessage::Progress).unwrap();
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

#[actix::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*places::NYC, 4);

    let mut operation = CurOperation::default();

    let (tx_load_observations, mut rx_load_observations) =
        tokio::sync::mpsc::unbounded_channel::<Observation>();
    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    ObservationLoaderActor::start_in_arbiter(&Arbiter::new().handle(), {
        let tx_app_message = tx_app_message.clone();
        let grid = grid.clone();
        |_ctx| ObservationLoaderActor {
            tx_app_message,
            tx_load_observations,
            grid,
        }
    });

    let image_store = sync::Arc::new(sync::RwLock::new(image_store::ImageStore::default()));

    let addr = ImageStoreActor::start_in_arbiter(&Arbiter::new().handle(), {
        let image_store = image_store.clone();
        |_| ImageStoreActor { image_store }
    });
    SystemRegistry::set(addr);

    let total_geohashes = grid.0.len();

    // FIXME: this thread never sleeps
    Arbiter::new().spawn(async move {
        while let Some(observation) = rx_load_observations.recv().await {
            // operation.visit_geohash_observations(geohash, &observations).await;
            operation
                .visit_observation(observation, tx_app_message.clone())
                .await
                .unwrap();
        }
    });

    eframe::run_native(
        "eframe template",
        eframe::NativeOptions::default(),
        Box::new(move |_| {
            Box::new(crate::app::TemplateApp {
                rx_app_message,
                loaded_geohashes: 0,
                total_geohashes,
                results: vec![],
                image_store,
            })
        }),
    )?;

    Ok(())
}
