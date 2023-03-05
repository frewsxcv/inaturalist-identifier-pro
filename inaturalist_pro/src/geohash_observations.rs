use crate::geohash_ext::Geohash;
use crate::Observation;
use futures::StreamExt;
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
        on_observation: impl Fn(Observation),
        soft_limit: &sync::atomic::AtomicI32,
        request: inaturalist::apis::observations_api::ObservationsGetParams,
    ) -> Result<(), FetchFromApiError> {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            tracing::info!("Hit soft limit.");
            return Ok(());
        }

        inaturalist_fetch::subdivide_rect(self.0.bounding_rect)
            .await
            .filter(|_| {
                if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
                    tracing::info!("Hit soft limit.");
                    return futures::future::ready(false);
                }
                futures::future::ready(true)
            })
            .then(|s| async {
                let rect = match s {
                    Ok(rect) => rect,
                    Err(e) => return Err(FetchFromApiError::INaturalistApi(e)),
                };
                match inaturalist_fetch::fetch(rect.0, |o| on_observation(o), soft_limit, request.clone())
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(FetchFromApiError::INaturalistApi(e)),
                }
            })
            .for_each(|_| futures::future::ready(()))
            .await;

        Ok(())
    }
}
