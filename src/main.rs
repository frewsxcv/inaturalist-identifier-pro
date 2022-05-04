use std::{collections, error, thread, time};

const PLANTAE_ID: u32 = 47126;

lazy_static::lazy_static! {
static ref INATURALIST_REQUEST_CONFIG: inaturalist::apis::configuration::Configuration =
    inaturalist::apis::configuration::Configuration {
        base_path: String::from("https://api.inaturalist.org/v1"),
        ..Default::default()
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let sw = geo_types::coord! { x: -74.046000f64, y: 40.567 };
    let ne = geo_types::coord! { x: -73.9389741f64, y: 40.6942535f64 };
    let rect = geo_types::Rect::new(sw, ne);

    let divisions = 32;

    let mut entries = vec![];

    for (i, rect) in grid_iter(rect, divisions).enumerate() {
        println!("Building new tile ({} / {})", i, divisions * divisions);
        let observations = fetch(rect).await?;

        entries.push((rect, observations_species_count(&observations)));

        thread::sleep(time::Duration::from_secs(1));
    }

    println!("{}", to_geojson(entries));

    Ok(())
}

type Entry = (geo_types::Rect<f64>, usize);
type Entries = Vec<Entry>;

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
    rect: geo_types::Rect<f64>,
) -> Result<
    Vec<inaturalist::models::Observation>,
    inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
> {
    let mut all = vec![];
    let mut page = 1;

    loop {
        println!("Fetching observations");
        let mut response = inaturalist::apis::observations_api::observations_get(
            &INATURALIST_REQUEST_CONFIG,
            build_params(rect, page),
        )
        .await?;

        all.append(&mut response.results);

        let last_page: u32 = (1 + response.total_results.unwrap() / response.per_page.unwrap())
            .try_into()
            .unwrap();

        if page == last_page {
            println!("No more pages (total results: {})", response.total_results.unwrap());
            break;
        } else {
            println!("New page");
        }

        page += 1;
    }

    Ok(all)
}

fn build_params(
    rect: geo_types::Rect<f64>,
    page: u32,
) -> inaturalist::apis::observations_api::ObservationsGetParams {
    inaturalist::apis::observations_api::ObservationsGetParams {
        swlat: Some(rect.min().y),
        swlng: Some(rect.min().x),
        nelat: Some(rect.max().y),
        nelng: Some(rect.max().x),
        quality_grade: Some(String::from("research")),
        taxon_id: Some(vec![PLANTAE_ID.to_string()]),
        per_page: Some(200.to_string()),
        native: Some(true),
        page: Some(page.to_string()),
        ..Default::default()
    }
}

fn grid_iter(
    rect: geo_types::Rect<f64>,
    divisions: u32,
) -> impl Iterator<Item = geo_types::Rect<f64>> {
    let grid_width = rect.width() / (divisions as f64);
    let grid_height = rect.height() / (divisions as f64);

    (0..(divisions * divisions)).map(move |n| {
        let x_offset = n % divisions;
        let y_offset = n / divisions;

        let sw_x = rect.min().x + (grid_width * (x_offset as f64));
        let sw_y = rect.min().y + (grid_height * (y_offset as f64));

        geo_types::Rect::new(
            geo_types::coord! { x: sw_x, y: sw_y, },
            geo_types::coord! { x: sw_x + grid_width, y: sw_y + grid_height, },
        )
    })
}
