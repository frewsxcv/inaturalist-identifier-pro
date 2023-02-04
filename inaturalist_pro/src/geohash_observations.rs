use crate::geohash_ext::Geohash;
use crate::rect_cache;
use crate::Observations;
use std::{env, path, sync, time};

#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    #[error("{0}")]
    FetchFromCache(#[from] FetchFromCacheError),
    #[error("{0}")]
    FetchFromApi(#[from] FetchFromApiError),
    #[error("{0}")]
    WriteToCache(#[from] WriteToCacheError),
}

#[derive(thiserror::Error, Debug)]
pub enum FetchFromApiError {
    #[error("{0}")]
    INaturalistApi(
        #[from] inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
    ),
}

#[derive(thiserror::Error, Debug)]
pub enum FetchFromCacheError {
    #[error("{0}")]
    TokioIo(#[from] tokio::io::Error),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum WriteToCacheError {
    #[error("{0}")]
    TokioIo(#[from] tokio::io::Error),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
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
        if let Ok(Some(observations)) = self.fetch_from_cache().await {
            soft_limit.fetch_sub(observations.len(), sync::atomic::Ordering::Relaxed);
            return Ok(observations);
        }

        let observations = self.fetch_from_api(soft_limit).await?;
        self.write_to_geohash_cache(&observations).await?;
        Ok(observations)
    }

    async fn fetch_from_cache(&self) -> Result<Option<Observations>, FetchFromCacheError> {
        let path = self.geohash_cache_path().await?;
        tracing::info!("Loading cache... ({})", path.display());
        if !path.exists() {
            tracing::info!("Cache not found");
            return Ok(None);
        }
        let file = tokio::fs::File::open(path).await?;
        let cache = serde_json::from_reader(file.into_std().await)?;
        tracing::info!("Fetched old cache");
        Ok(Some(cache))
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

            let mut fetched = match rect_cache::fetch(s.0).await.unwrap() {
                // TODO no unwrap
                Some(cached) => cached,
                None => {
                    let fetched = inaturalist_fetch::fetch(s.0).await?;
                    rect_cache::write(s.0, &fetched).await.unwrap(); // TODO no unwrap
                    fetched
                }
            };

            soft_limit.fetch_sub(observations.len(), sync::atomic::Ordering::Relaxed);

            observations.append(&mut fetched);
        }
        Ok(observations)
    }

    async fn geohash_cache_dir() -> tokio::io::Result<path::PathBuf> {
        let path = env::temp_dir().join("inaturalist-geohash-request-cache");
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await?;
        }
        Ok(path)
    }

    async fn geohash_cache_path(&self) -> tokio::io::Result<path::PathBuf> {
        Ok(Self::geohash_cache_dir()
            .await?
            .join(self.0.string.as_str()))
    }

    async fn write_to_geohash_cache(
        &self,
        observations: &Observations,
    ) -> Result<(), WriteToCacheError> {
        let cache_path = self.geohash_cache_path().await?;
        let file = tokio::fs::File::create(cache_path).await?;
        tracing::info!("Writing cache...");
        serde_json::to_writer(file.into_std().await, &observations)?;
        tracing::info!("done");
        Ok(())
    }
}
