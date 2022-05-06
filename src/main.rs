use geo::algorithm::contains::Contains;
use std::hash::{Hash, Hasher};
use std::{collections, error, fs};

const PLANTAE_ID: u32 = 47126;

type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;

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
        governor::RateLimiter::direct(
            governor::Quota::per_second(1.try_into().unwrap()),
        );
    static ref INATURALIST_REQUEST_CACHE: async_mutex::Mutex<RequestCache> =
        async_mutex::Mutex::new(RequestCache::load_or_create());
}

// TODO: read and write from request cache
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // Brooklyn
    // let sw = geo::coord! { x: -74.046000f64, y: 40.567 };
    // let ne = geo::coord! { x: -73.9389741f64, y: 40.6942535f64 };

    let sw = geo::coord! {
        x: ordered_float::OrderedFloat(-74.258019),
        y: ordered_float::OrderedFloat(40.490742)
    };
    let ne = geo::coord! {
        x: ordered_float::OrderedFloat(-73.555615),
        y: ordered_float::OrderedFloat(41.017433)
    };

    let rect = geo::Rect::new(sw, ne);

    let divisions = 32;

    let mut entries = vec![];

    let subdivided_rects = subdivide_rect(rect).await?;
    let num_rects = subdivided_rects.len();
    let mut observations = vec![];
    for (i, s) in subdivided_rects.into_iter().enumerate() {
        println!("Fetch tile ({} / {})", i, num_rects);
        observations.append(&mut fetch(s.0).await?);
    }

    for (i, rect) in grid_iter(rect, divisions).enumerate() {
        println!("Building new tile ({} / {})", i, divisions * divisions);
        let mut observations_in_tile = vec![];

        for observation in &observations {
            if let Some(c) = observation
                .geojson
                .as_ref()
                .and_then(|g| g.coordinates.as_ref())
            {
                if rect.contains(&geo::point! { x: ordered_float::OrderedFloat(c[0]), y: ordered_float::OrderedFloat(c[1]) }) {
                    observations_in_tile.push(observation.clone());
                }
            }
        }

        entries.push((rect, observations_species_count(&observations_in_tile)));
    }

    fs::write(
        "/Users/coreyf/tmp/output.geojson",
        to_geojson(entries).to_string(),
    )?;

    Ok(())
}

struct SubdividedRect(Rect);

type Entry = (Rect, usize);
type Entries = Vec<Entry>;

/// iNaturalist will not let us page past 10,000 results.
const MAX_RESULTS: i32 = 10_000;

#[async_recursion::async_recursion]
async fn subdivide_rect(
    rect: Rect,
) -> Result<
    Vec<SubdividedRect>,
    inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
> {
    let page = 1;
    let per_page = 1;
    let response = {
        let mut request_cache = INATURALIST_REQUEST_CACHE.lock().await;
        if let Some(response) = request_cache.get(rect, per_page, page) {
            println!("Found in cache");
            response.clone()
        } else {
            println!("Not found in cache");
            INATURALIST_RATE_LIMITER.until_ready().await;
            let response = inaturalist::apis::observations_api::observations_get(
                &INATURALIST_REQUEST_CONFIG,
                build_params(rect, page, per_page),
            )
            .await?;
            request_cache.insert(rect, per_page, page, response.clone());
            response
        }
    };

    Ok(if response.total_results.unwrap() < MAX_RESULTS {
        println!("Rect is sufficient");
        vec![SubdividedRect(rect)]
    } else {
        println!(
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
    observations
        .iter()
        .filter_map(|observation| observation.taxon.as_ref())
        .map(|taxon| taxon.id)
        .collect::<collections::HashSet<_>>()
        .len()
}

fn to_geojson(entries: Entries) -> geojson::FeatureCollection {
    let mut features = vec![];
    for entry in entries {
        let value = geojson::Value::try_from(&entry.0.to_polygon()).unwrap();
        let mut properties = serde_json::Map::new();
        properties.insert("amount".into(), entry.1.into());
        features.push(geojson::Feature {
            geometry: Some(value.into()),
            properties: Some(properties),
            bbox: None,
            id: None,
            foreign_members: None,
        })
    }
    geojson::FeatureCollection {
        features,
        bbox: None,
        foreign_members: None,
    }
}

async fn fetch(
    rect: Rect,
) -> Result<
    Vec<inaturalist::models::Observation>,
    inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
> {
    let mut all = vec![];
    let mut page = 1;
    let per_page = 200;

    loop {
        let mut response = {
            let mut request_cache = INATURALIST_REQUEST_CACHE.lock().await;
            if let Some(response) = request_cache.get(rect, per_page, page) {
                println!("Fetched observations from cache");
                response.clone()
            } else {
                println!("Fetching observations");
                INATURALIST_RATE_LIMITER.until_ready().await;
                let response = inaturalist::apis::observations_api::observations_get(
                    &INATURALIST_REQUEST_CONFIG,
                    build_params(rect, page, per_page),
                )
                .await?;
                request_cache.insert(rect, per_page, page, response.clone());
                response
            }
        };

        all.append(&mut response.results);

        let per_page = response.per_page.unwrap() as u32;
        let total_results = response.total_results.unwrap() as u32;

        let last_page: u32 = 1 + total_results / per_page;

        if page == last_page {
            println!(
                "No more pages (total results: {})",
                response.total_results.unwrap()
            );
            break;
        } else {
            println!(
                "New page ({} / {} | {})",
                per_page * page,
                total_results,
                (per_page as f32 * page as f32) / (total_results as f32)
            );
        }

        page += 1;
    }

    Ok(all)
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RequestCache(collections::HashMap<String, inaturalist::models::ObservationsResponse>);

impl RequestCache {
    fn load_or_create() -> Self {
        Self::load().unwrap_or_else(|| {
            println!("Creating new cache");
            RequestCache(collections::HashMap::new())
        })
    }

    fn load() -> Option<Self> {
        let file = fs::File::open("/tmp/inaturalist-request-cache.json").ok()?;
        let cache = serde_json::from_reader(file).ok()?;
        println!("Fetched old cache");
        cache
    }

    fn get(
        &self,
        rect: Rect,
        per_page: u32,
        page: u32,
    ) -> Option<&inaturalist::models::ObservationsResponse> {
        self.0.get(&hash_request_info(rect, per_page, page))
    }

    fn insert(
        &mut self,
        rect: Rect,
        per_page: u32,
        page: u32,
        response: inaturalist::models::ObservationsResponse,
    ) {
        self.0
            .insert(hash_request_info(rect, per_page, page), response);
        let file = fs::File::create("/tmp/inaturalist-request-cache.json").unwrap();
        print!("Writing cache...");
        serde_json::to_writer(file, &self.0).unwrap();
        println!("done");
    }
}

fn hash_request_info(rect: Rect, per_page: u32, page: u32) -> String {
    let mut hasher = collections::hash_map::DefaultHasher::new();
    rect.hash(&mut hasher);
    per_page.hash(&mut hasher);
    page.hash(&mut hasher);
    format!("{:x}", hasher.finish())
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
        native: Some(true),
        page: Some(page.to_string()),
        ..Default::default()
    }
}

fn grid_iter(rect: Rect, divisions: u32) -> impl Iterator<Item = Rect> {
    let grid_width = rect.width() / (divisions as f64);
    let grid_height = rect.height() / (divisions as f64);

    (0..(divisions * divisions)).map(move |n| {
        let x_offset = n % divisions;
        let y_offset = n / divisions;

        let sw_x = rect.min().x + (grid_width * (x_offset as f64));
        let sw_y = rect.min().y + (grid_height * (y_offset as f64));

        geo::Rect::new(
            geo::coord! { x: sw_x, y: sw_y, },
            geo::coord! { x: sw_x + grid_width, y: sw_y + grid_height, },
        )
    })
}
