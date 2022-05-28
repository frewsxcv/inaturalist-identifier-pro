use geo::algorithm::contains::Contains;
use geohash_ext::{Geohash, GeohashGrid};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::{collections, env, error, fs, io, num, path, process};

mod geo_ext;
mod geohash_ext;

const PLANTAE_ID: u32 = 47126;

const INATURALIST_RATE_LIMIT_AMOUNT: governor::Quota =
    governor::Quota::per_second(unsafe { num::NonZeroU32::new_unchecked(1) });

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;
type Observations = Vec<inaturalist::models::Observation>;

lazy_static::lazy_static! {
    static ref INATURALIST_REQUEST_CONFIG: inaturalist::apis::configuration::Configuration =
        inaturalist::apis::configuration::Configuration {
            base_path: String::from("https://api.inaturalist.org/v1"),
            ..Default::default()
        };

    static ref INATURALIST_RATE_LIMITER: governor::RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    > =
        governor::RateLimiter::direct(INATURALIST_RATE_LIMIT_AMOUNT);

    static ref HARRIMAN_STATE_PARK: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-74.26345825195312),
            y: ordered_float::OrderedFloat(41.101086483800515),
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.948873),
            y: ordered_float::OrderedFloat(41.34124700339191)
        },
    );

    static ref BROOKLYN: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-74.046000f64),
            y: ordered_float::OrderedFloat(40.567),
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.9389741f64),
            y: ordered_float::OrderedFloat(40.6942535f64),
        },
    );

    static ref NYC: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-74.258019),
            y: ordered_float::OrderedFloat(40.490742)
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.555615),
            y: ordered_float::OrderedFloat(41.017433)
        },
    );
}

struct GeohashObservations(Geohash);

impl GeohashObservations {
    async fn fetch(&self) -> Result<Observations, Box<dyn error::Error>> {
        if let Ok(Some(observations)) = self.fetch_from_cache() {
            return Ok(observations);
        }

        let observations = self.fetch_from_api().await?;
        self.write_to_cache(&observations)?;
        Ok(observations)
    }

    fn fetch_from_cache(&self) -> Result<Option<Observations>, Box<dyn error::Error>> {
        let path = self.cache_path()?;
        tracing::info!("Loading cache... ({})", path.display());
        if !path.exists() {
            return Ok(None);
        }
        let file = fs::File::open(path)?;
        let cache = serde_json::from_reader(file)?;
        tracing::info!("Fetched old cache");
        Ok(Some(cache))
    }

    async fn fetch_from_api(&self) -> Result<Observations, Box<dyn error::Error>> {
        let subdivided_rects = subdivide_rect(self.0.bounding_rect).await?;
        let num_rects = subdivided_rects.len();
        let mut observations = Vec::with_capacity(subdivided_rects.len());
        for (i, s) in subdivided_rects.into_iter().enumerate() {
            tracing::info!("Fetch tile ({} / {})", i + 1, num_rects);
            observations.append(&mut fetch(s.0).await?);
        }
        Ok(observations)
    }

    fn cache_dir() -> Result<path::PathBuf, Box<dyn error::Error>> {
        let path = env::temp_dir().join("inaturalist-request-cache");
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    fn cache_path(&self) -> Result<path::PathBuf, Box<dyn error::Error>> {
        Ok(Self::cache_dir()?.join(&self.0.string))
    }

    fn write_to_cache(&self, observations: &Observations) -> Result<(), Box<dyn error::Error>> {
        let file = fs::File::create(self.cache_path()?)?;
        tracing::info!("Writing cache...");
        let _ = io::stdout().flush();
        serde_json::to_writer(file, &observations)?;
        tracing::info!("done");
        let _ = io::stdout().flush();
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    tracing_subscriber::fmt::init();

    let grid = GeohashGrid::from_rect(*HARRIMAN_STATE_PARK, 5);
    let grid_count = grid.0.len();
    let mut geojson_features = vec![];

    for (i, geohash) in grid.0.into_iter().enumerate() {
        tracing::info!(
            "Fetch observations for geohash {} ({} / {})",
            geohash.string,
            i + 1,
            grid_count
        );
        let mut geojson_feature = geohash.to_geojson_feature();
        let observations = GeohashObservations(geohash).fetch().await?;
        let species_count = observations_species_count(&observations);
        if let Some(properties) = &mut geojson_feature.properties {
            properties.insert("species count".into(), species_count.into());
        }
        geojson_features.push(geojson_feature);
    }

    let geojson_feature_collection = geojson::FeatureCollection {
        features: geojson_features,
        bbox: None,
        foreign_members: None,
    };

    fs::write(
        "/Users/coreyf/tmp/output.geojson",
        geojson_feature_collection.to_string(),
    )?;

    process::exit(0);
}

struct SubdividedRect(Rect);

/// iNaturalist will not let us page past 10,000 results.
const MAX_RESULTS: i32 = 10_000;

const MAX_RESULTS_PER_PAGE: u32 = 200;

#[async_recursion::async_recursion]
async fn subdivide_rect(
    rect: Rect,
) -> Result<
    Vec<SubdividedRect>,
    inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
> {
    let page = 1;
    let per_page = 1;
    INATURALIST_RATE_LIMITER.until_ready().await;

    let response = inaturalist::apis::observations_api::observations_get(
        &INATURALIST_REQUEST_CONFIG,
        build_params(rect, page, per_page),
    )
    .await?;

    Ok(if response.total_results.unwrap() < MAX_RESULTS {
        tracing::info!("Rect is sufficient");
        vec![SubdividedRect(rect)]
    } else {
        tracing::info!(
            "Splitting rect (total_results: {})",
            response.total_results.unwrap()
        );
        let (rect1, rect2) = split_rect(rect);
        let mut s1 = subdivide_rect(rect1).await?;
        let mut s2 = subdivide_rect(rect2).await?;
        s1.append(&mut s2);
        s1
    })
}

fn split_rect(rect: Rect) -> (Rect, Rect) {
    if rect.width() > rect.height() {
        let mid = rect.min().x + rect.width() / 2.;
        (
            geo::Rect::new(
                geo::coord! { x: rect.min().x, y: rect.min().y },
                geo::coord! { x: mid, y: rect.max().y },
            ),
            geo::Rect::new(
                geo::coord! { x: mid, y: rect.min().y },
                geo::coord! { x: rect.max().x, y: rect.max().y },
            ),
        )
    } else {
        let mid = rect.min().y + rect.height() / 2.;
        (
            geo::Rect::new(
                geo::coord! { x: rect.min().x, y: rect.min().y },
                geo::coord! { x: rect.max().x, y: mid },
            ),
            geo::Rect::new(
                geo::coord! { x: rect.min().x, y: mid },
                geo::coord! { x: rect.max().x, y: rect.max().y },
            ),
        )
    }
}

fn observations_species_count(observations: &[inaturalist::models::Observation]) -> usize {
    // TODO this should actually be a ratio?
    observations
        .iter()
        .filter_map(|observation| observation.taxon.as_ref())
        .map(|taxon| taxon.id)
        .collect::<collections::HashSet<_>>()
        .len()
}

async fn fetch(
    rect: Rect,
) -> Result<
    Vec<inaturalist::models::Observation>,
    inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
> {
    let mut all = vec![];
    let per_page = MAX_RESULTS_PER_PAGE;

    for page in 1.. {
        tracing::info!("Fetching observations...");
        let _ = io::stdout().flush();
        INATURALIST_RATE_LIMITER.until_ready().await;
        let mut response = inaturalist::apis::observations_api::observations_get(
            &INATURALIST_REQUEST_CONFIG,
            build_params(rect, page, per_page),
        )
        .await?;
        tracing::info!("done");
        let _ = io::stdout().flush();

        all.append(&mut response.results);

        let per_page = response.per_page.unwrap() as u32;
        let total_results = response.total_results.unwrap() as u32;

        let last_page: u32 = 1 + total_results / per_page;

        if page == last_page {
            tracing::info!(
                "No more pages (total results: {})",
                response.total_results.unwrap()
            );
            break;
        } else {
            tracing::info!(
                "New page ({} / {} | {})",
                per_page * page,
                total_results,
                (per_page as f32 * page as f32) / (total_results as f32)
            );
        }
    }

    Ok(all)
}

fn build_params(
    rect: Rect,
    page: u32,
    per_page: u32,
) -> inaturalist::apis::observations_api::ObservationsGetParams {
    inaturalist::apis::observations_api::ObservationsGetParams {
        swlat: Some(*rect.min().y),
        swlng: Some(*rect.min().x),
        nelat: Some(*rect.max().y),
        nelng: Some(*rect.max().x),
        // quality_grade: Some(String::from("research")),
        captive: Some(false),
        taxon_id: Some(vec![PLANTAE_ID.to_string()]),
        per_page: Some(per_page.to_string()),
        identified: Some(true),
        identifications: Some(String::from("most_agree")),
        native: Some(true),
        page: Some(page.to_string()),
        ..Default::default()
    }
}
