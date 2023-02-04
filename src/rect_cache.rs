use crate::Observations;
use std::{
    collections, env,
    hash::{Hash, Hasher},
    io::{self, Write},
    path,
};

#[derive(thiserror::Error, Debug)]
pub enum FetchFromCacheError {
    #[error("{0}")]
    TokioIo(#[from] tokio::io::Error),
    #[error("{0}")]
    Bincode(#[from] bincode::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum WriteToCacheError {
    #[error("{0}")]
    TokioIo(#[from] tokio::io::Error),
    #[error("{0}")]
    Bincode(#[from] bincode::Error),
}

pub async fn fetch(
    rect: geo::Rect<ordered_float::OrderedFloat<f64>>,
) -> Result<Option<Observations>, FetchFromCacheError> {
    let path = path(rect).await?;
    tracing::info!("Loading cache... ({})", path.display());
    if !path.exists() {
        return Ok(None);
    }
    let file = tokio::fs::File::open(path).await?;
    let cache = bincode::deserialize_from(file.into_std().await)?;
    tracing::info!("Fetched old cache");
    Ok(Some(cache))
}

pub async fn write(
    rect: geo::Rect<ordered_float::OrderedFloat<f64>>,
    observations: &Observations,
) -> Result<(), WriteToCacheError> {
    let cache_path = path(rect).await?;
    let file = tokio::fs::File::create(cache_path).await?;
    tracing::info!("Writing cache...");
    let _ = io::stdout().flush();
    bincode::serialize_into(file.into_std().await, &observations)?;
    tracing::info!("done");
    let _ = io::stdout().flush();
    Ok(())
}

pub async fn dir() -> tokio::io::Result<path::PathBuf> {
    let path = env::temp_dir().join("inaturalist-rect-request-cache");
    if !path.exists() {
        tokio::fs::create_dir_all(&path).await?;
    }
    Ok(path)
}

pub async fn path(
    rect: geo::Rect<ordered_float::OrderedFloat<f64>>,
) -> tokio::io::Result<path::PathBuf> {
    let mut hasher = collections::hash_map::DefaultHasher::new();
    rect.hash(&mut hasher);
    let hash = format!("{}", hasher.finish());
    Ok(dir().await?.join(&hash))
}
