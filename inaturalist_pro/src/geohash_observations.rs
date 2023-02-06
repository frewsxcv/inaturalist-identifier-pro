use crate::geohash_ext::Geohash;
use crate::Observation;
use std::sync;

#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    #[error("{0}")]
    FetchFromApi(#[from] FetchFromApiError),
}

#[derive(thiserror::Error, Debug)]
pub enum FetchFromApiError {
    #[error("{0}")]
    INaturalistApi(
        #[from] inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
    ),
}

pub struct GeohashObservations(pub Geohash);

impl GeohashObservations {
    pub async fn fetch_from_api(
        &self,
        tx: tokio::sync::mpsc::UnboundedSender<Observation>,
        soft_limit: &sync::atomic::AtomicI32,
    ) -> Result<(), FetchFromApiError> {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            tracing::info!("Hit soft limit.");
            return Ok(());
        }

        let subdivided_rects = inaturalist_fetch::subdivide_rect(self.0.bounding_rect).await?;
        let num_rects = subdivided_rects.len();
        for (i, s) in subdivided_rects.into_iter().enumerate() {
            if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
                tracing::info!("Hit soft limit.");
                break;
            }

            tracing::info!("Fetch tile ({} / {})", i + 1, num_rects);

            inaturalist_fetch::fetch(s.0, tx.clone(), soft_limit).await?;
        }
        Ok(())
    }
}
