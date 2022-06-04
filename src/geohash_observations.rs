use crate::geohash_ext::Geohash;
use crate::Observations;
use std::{env, io::{self, Write}, error, path};

pub struct GeohashObservations(pub Geohash);

impl GeohashObservations {
    pub async fn fetch(&self) -> Result<Observations, Box<dyn error::Error>> {
        if let Ok(Some(observations)) = self.fetch_from_cache().await {
            return Ok(observations);
        }

        let observations = self.fetch_from_api().await?;
        self.write_to_cache(&observations).await?;
        Ok(observations)
    }

    async fn fetch_from_cache(&self) -> Result<Option<Observations>, Box<dyn error::Error>> {
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

    async fn fetch_from_api(&self) -> Result<Observations, Box<dyn error::Error>> {
        let subdivided_rects = crate::subdivide_rect(self.0.bounding_rect).await?;
        let num_rects = subdivided_rects.len();
        let mut observations = Vec::with_capacity(subdivided_rects.len());
        for (i, s) in subdivided_rects.into_iter().enumerate() {
            tracing::info!("Fetch tile ({} / {})", i + 1, num_rects);
            observations.append(&mut crate::fetch(s.0).await?);
        }
        Ok(observations)
    }

    async fn cache_dir() -> Result<path::PathBuf, Box<dyn error::Error>> {
        let path = env::temp_dir().join("inaturalist-request-cache");
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await?;
        }
        Ok(path)
    }

    async fn cache_path(&self) -> Result<path::PathBuf, Box<dyn error::Error>> {
        Ok(Self::cache_dir().await?.join(&self.0.string))
    }

    async fn write_to_cache(
        &self,
        observations: &Observations,
    ) -> Result<(), Box<dyn error::Error>> {
        let file = tokio::fs::File::create(self.cache_path().await?).await?;
        tracing::info!("Writing cache...");
        let _ = io::stdout().flush();
        serde_json::to_writer(file.into_std().await, &observations)?;
        tracing::info!("done");
        let _ = io::stdout().flush();
        Ok(())
    }
}
