use crate::geohash_ext::Geohash;
use crate::Observations;
use std::{sync, time};

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
    pub async fn fetch_with_retries(&self, soft_limit: &sync::atomic::AtomicUsize) -> Observations {
        let observations;
        loop {
            match GeohashObservations(self.0).fetch(soft_limit).await {
                Ok(o) => {
                    observations = o;
                    break;
                }
                Err(_) => {
                    tracing::info!("Encountered an error when fetching. Trying again. {:?}", (),);
                    tokio::time::sleep(time::Duration::from_secs(5)).await;
                    continue;
                }
            };
        }
        observations
    }

    pub async fn fetch(
        &self,
        soft_limit: &sync::atomic::AtomicUsize,
    ) -> Result<Observations, FetchError> {
        let observations = self.fetch_from_api(soft_limit).await?;
        Ok(observations)
    }

    async fn fetch_from_api(
        &self,
        soft_limit: &sync::atomic::AtomicUsize,
    ) -> Result<Observations, FetchFromApiError> {
        let subdivided_rects = inaturalist_fetch::subdivide_rect(self.0.bounding_rect).await?;
        let num_rects = subdivided_rects.len();
        let mut observations = Vec::with_capacity(subdivided_rects.len());
        for (i, s) in subdivided_rects.into_iter().enumerate() {
            tracing::info!("Fetch tile ({} / {})", i + 1, num_rects);

            if observations.len() > soft_limit.load(sync::atomic::Ordering::Relaxed) {
                break;
            }

            let mut fetched = inaturalist_fetch::fetch(s.0).await?;

            soft_limit.fetch_sub(observations.len(), sync::atomic::Ordering::Relaxed);

            observations.append(&mut fetched);
        }
        Ok(observations)
    }
}
