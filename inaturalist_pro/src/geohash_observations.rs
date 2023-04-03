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
        on_observation: impl Fn(Observation),
        soft_limit: &sync::atomic::AtomicI32,
        request: inaturalist::apis::observations_api::ObservationsGetParams,
    ) -> Result<(), FetchFromApiError> {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            return Ok(());
        }

        let mut gen = inaturalist_fetch::subdivide_rect_iter(self.0.bounding_rect, request.clone());
        while let genawaiter::GeneratorState::Yielded(result) = gen.async_resume().await {
            tracing::info!("Yielding: {:?}", result);
            // tracing::info!("Received new observations");
            /*
            tracing::info!("FILTER");
            if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
                return futures::future::ready(false);
            }
            futures::future::ready(true)
            */

            let rect = match result {
                Ok(rect) => rect,
                Err(e) => return Err(FetchFromApiError::INaturalistApi(e)),
            };

            match inaturalist_fetch::fetch(
                rect.0,
                #[allow(clippy::redundant_closure)]
                |o| on_observation(o),
                soft_limit,
                request.clone(),
            )
            .await
            {
                Ok(_) => (),
                Err(e) => return Err(FetchFromApiError::INaturalistApi(e)),
            }
        }

        Ok(())
    }
}
