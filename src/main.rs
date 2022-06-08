use geohash_ext::{Geohash, GeohashGrid};
use geohash_observations::GeohashObservations;
use inaturalist::models::Observation;
use operations::Operation;
use std::{error, sync};

mod app;
mod fetch;
mod geohash_ext;
mod geohash_observations;
mod operations;
mod places;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;
type Observations = Vec<Observation>;

#[derive(Debug)]
struct Progress(f32);

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*places::HARRIMAN_STATE_PARK, 5);
    let grid_count = grid.0.len();

    let operation = sync::Arc::new(tokio::sync::Mutex::new(operations::PrintPlantae::default()));
    // let mut operation = operations::GeoJsonUniqueSpecies { geojson_features: vec![] };

    let (tx, rx) = async_channel::unbounded::<Progress>();

    let total_geohashes = grid.0.len();

    tokio::task::spawn(async move {
        let mut join_handles = vec![];
        for (i, geohash) in grid.0.into_iter().enumerate() {
            let operation = operation.clone();
            let tx = tx.clone();
            join_handles.push(tokio::spawn(async move {
                let geohash = geohash.clone();
                tracing::info!(
                    "Fetch observations for geohash {} ({} / {})",
                    geohash.string,
                    i + 1,
                    grid_count
                );
                let observations = GeohashObservations(geohash.clone()).fetch().await.unwrap();
                {
                    let mut lock = operation.lock().await;
                    lock.visit_geohash_observations(&geohash, &observations);
                    for observation in observations {
                        lock.visit_observation(&observation);
                    }
                }
                tx.send(Progress(i as f32)).await.unwrap();
            }));
        }
        for join_handle in join_handles {
            join_handle.await.unwrap();
        }
        operation.lock().await.finish();
    });

    let native_options = eframe::NativeOptions::default();
    let urls = vec![];
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(move |_| Box::new(crate::app::TemplateApp {
            display_string: urls,
            rx_progress: rx,
            loaded_geohashes: 0,
            total_geohashes,
        })),
    );
}
