use crate::geohash_ext::Geohash;
use crate::Observations;
use std::{
    env,
    io::{self, Write},
    path,
};

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
    pub async fn fetch(&self) -> Result<Observations, FetchError> {
        if let Ok(Some(observations)) = self.fetch_from_cache().await {
            return Ok(observations);
        }

        let observations = self.fetch_from_api().await?;
        self.write_to_cache(&observations).await?;
        Ok(observations)
    }

    async fn fetch_from_cache(&self) -> Result<Option<Observations>, FetchFromCacheError> {
        let path = self.cache_path().await?;
        tracing::info!("Loading cache... ({})", path.display());
        if !path.exists() {
            return Ok(None);
        }
        let file = tokio::fs::File::open(path).await?;
        let cache = serde_json::from_reader(file.into_std().await)?;
        tracing::info!("Fetched old cache");
        Ok(Some(cache))
    }

    async fn fetch_from_api(&self) -> Result<Observations, FetchFromApiError> {
        let subdivided_rects = crate::fetch::subdivide_rect(self.0.bounding_rect).await?;
        let num_rects = subdivided_rects.len();
        let mut observations = Vec::with_capacity(subdivided_rects.len());
        for (i, s) in subdivided_rects.into_iter().enumerate() {
            tracing::info!("Fetch tile ({} / {})", i + 1, num_rects);
            observations.append(&mut crate::fetch::fetch(s.0).await?);
        }
        Ok(observations)
    }

    async fn cache_dir() -> tokio::io::Result<path::PathBuf> {
        let path = env::temp_dir().join("inaturalist-request-cache");
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await?;
        }
        Ok(path)
    }

    async fn cache_path(&self) -> tokio::io::Result<path::PathBuf> {
        Ok(Self::cache_dir().await?.join(&self.0.string))
    }

    async fn write_to_cache(&self, observations: &Observations) -> Result<(), WriteToCacheError> {
        let cache_path = self.cache_path().await?;
        let file = tokio::fs::File::create(cache_path).await?;
        tracing::info!("Writing cache...");
        let _ = io::stdout().flush();
        serde_json::to_writer(file.into_std().await, &observations)?;
        tracing::info!("done");
        let _ = io::stdout().flush();
        Ok(())
    }
}
