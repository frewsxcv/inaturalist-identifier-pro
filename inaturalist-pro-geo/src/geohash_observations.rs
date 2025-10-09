use crate::geohash_ext::Geohash;
use inaturalist::models::Observation;
use std::error::Error;
use std::sync;

#[derive(Debug)]
pub enum FetchFromApiError {
    INaturalistApi(
        inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
    ),
}

impl Error for FetchFromApiError {}

impl std::fmt::Display for FetchFromApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchFromApiError::INaturalistApi(e) => write!(f, "iNaturalist API error: {e}"),
        }
    }
}

pub struct GeohashObservations(pub Geohash);

impl GeohashObservations {
    pub async fn fetch_from_api(
        &self,
        on_observation: impl Fn(Observation),
        soft_limit: &sync::atomic::AtomicI32,
        request: inaturalist::apis::observations_api::ObservationsGetParams,
        api_token: &str,
    ) -> Result<(), FetchFromApiError> {
        if soft_limit.load(sync::atomic::Ordering::Relaxed) < 0 {
            return Ok(());
        }

        let mut gen = inaturalist_fetch::subdivide_rect_iter(
            self.0.bounding_rect,
            request.clone(),
            api_token.to_string(),
        );
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
                api_token,
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
