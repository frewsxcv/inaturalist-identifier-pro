#![feature(async_fn_in_trait)]
#![feature(async_closure)]

use geohash_ext::GeohashGrid;
use geohash_observations::GeohashObservations;
use inaturalist::models::Observation;
use operations::Operation;
use std::{error, sync};

mod app;
mod geohash_ext;
mod geohash_observations;
mod image_store;
mod operations;
mod places;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

#[derive(Debug)]
pub enum AppMessage {
    Progress,
    Result((Box<Observation>, f32)),
}

lazy_static::lazy_static! {
    static ref FETCH_SOFT_LIMIT: sync::atomic::AtomicI32 = sync::atomic::AtomicI32::new(500);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*places::NYC, 4);
    let grid_count = grid.0.len();

    type Operation = operations::TopImageScore;

    let mut operation = Operation::default();

    let (tx_load_observations, mut rx_load_observations) =
        tokio::sync::mpsc::unbounded_channel::<Observation>();
    let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    let total_geohashes = grid.0.len();

    let foo = tx_app_message.clone();

    // FIXME: this thread never sleeps
    tokio::task::spawn(async move {
        while let Some(observation) = rx_load_observations.recv().await {
            // operation.visit_geohash_observations(geohash, &observations).await;
            operation
                .visit_observation(observation, foo.clone())
                .await
                .unwrap();
        }
    });

    tokio::task::spawn(async move {
        let tx_app_message = tx_app_message.clone();
        for (i, geohash) in grid.0.into_iter().enumerate() {
            let tx_app_message = tx_app_message.clone();
            tracing::info!(
                "Fetch observations for geohash {} ({} / {})",
                geohash.string,
                i + 1,
                grid_count
            );
            GeohashObservations(geohash)
                .fetch_from_api(
                    tx_load_observations.clone(),
                    &FETCH_SOFT_LIMIT,
                    Operation::request(),
                )
                .await
                .unwrap();
            tx_app_message.send(AppMessage::Progress).unwrap();
        }
        // FIXME: call below
        // operation.lock().await.finish();
        tracing::info!("Finished loading thread");
        // tx.send(AppMessage::Results(std::mem::take(
        //     &mut operation.0,
        // )))
        // .await
        // .unwrap();
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
                image_store: sync::Arc::new(sync::RwLock::new(image_store::ImageStore::default())),
            })
        }),
    );

    Ok(())
}
