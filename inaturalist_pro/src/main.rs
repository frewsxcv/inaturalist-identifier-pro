use geohash_ext::{Geohash, GeohashGrid};
use geohash_observations::GeohashObservations;
use inaturalist::models::Observation;
use operations::Operation;
use std::{error, sync};

mod app;
mod fetch;
mod geohash_ext;
mod geohash_observations;
mod image_store;
mod operations;
mod places;
mod rect_cache;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;
type Observations = Vec<Observation>;

#[derive(Debug)]
enum AppMessage {
    Progress,
    Results(Vec<Observation>),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*places::HARRIMAN_STATE_PARK, 4);
    let grid_count = grid.0.len();

    // let operation = sync::Arc::new(tokio::sync::Mutex::new(operations::TopObservationsPerTile::default()));
    // let operation = sync::Arc::new(tokio::sync::Mutex::new(operations::PrintPlantae::default()));
    let operation = sync::Arc::new(tokio::sync::Mutex::new(
        operations::PrintAngiospermae::default(),
    ));
    // let mut operation = operations::GeoJsonUniqueSpecies { geojson_features: vec![] };

    let (tx, rx_app_message) = async_channel::unbounded::<AppMessage>();

    let total_geohashes = grid.0.len();

    tokio::task::spawn(async move {
        let mut join_handles = vec![];
        for (i, geohash) in grid.0.into_iter().enumerate() {
            let operation = operation.clone();
            let tx = tx.clone();
            join_handles.push(tokio::spawn(async move {
                tracing::info!(
                    "Fetch observations for geohash {} ({} / {})",
                    geohash.string,
                    i + 1,
                    grid_count
                );
                let observations = GeohashObservations(geohash).fetch_with_retries().await;
                {
                    let mut lock = operation.lock().await;
                    lock.visit_geohash_observations(geohash, &observations);
                    for observation in observations {
                        lock.visit_observation(&observation);
                    }
                }
                tx.send(AppMessage::Progress).await.unwrap();
            }));
        }
        for join_handle in join_handles {
            join_handle.await.unwrap();
        }
        operation.lock().await.finish();
        tracing::info!("Finished loading thread");
        tx.send(AppMessage::Results(std::mem::take(
            &mut operation.lock().await.0,
        )))
        .await
        .unwrap();
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
}
