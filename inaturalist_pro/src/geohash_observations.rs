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
    pub async fn fetch_with_retries(&self, soft_limit: &sync::atomic::AtomicI32) -> Observations {
        let observations;
        loop {
            match GeohashObservations(self.0).fetch_from_api(soft_limit).await {
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

    async fn fetch_from_api(
        &self,
        soft_limit: &sync::atomic::AtomicI32,
    ) -> Result<Observations, FetchFromApiError> {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            return Ok(vec![]);
        }

        let subdivided_rects = inaturalist_fetch::subdivide_rect(self.0.bounding_rect).await?;
        let num_rects = subdivided_rects.len();
        let mut observations = Vec::with_capacity(subdivided_rects.len());
        for (i, s) in subdivided_rects.into_iter().enumerate() {
            if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
                tracing::info!("Hit soft limit.");
                break;
            }

            tracing::info!("Fetch tile ({} / {})", i + 1, num_rects);

            let mut fetched = inaturalist_fetch::fetch(s.0, soft_limit).await?;

            tracing::info!("fetched: {}", soft_limit.load(sync::atomic::Ordering::Relaxed));

            observations.append(&mut fetched);
        }
        Ok(observations)
    }
}
