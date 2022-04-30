use std::{error, thread, time};

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
    let sw = geo_types::coord! { x: -74.046000, y: 40.600007 };
    let ne = geo_types::coord! { x: -73.9389741, y: 40.6942535 };
    let rect = geo_types::Rect::new(sw, ne);

    let divisions = 2;

    for rect in grid_iter(rect, divisions) {
        let observations = fetch(rect).await?;

        println!("{:#?}", observations.results);

        thread::sleep(time::Duration::from_secs(1));
    }

    // todo: output geojson grid

    Ok(())
}

async fn fetch(
    rect: geo_types::Rect<f64>,
) -> Result<
    inaturalist::models::ObservationsResponse,
    inaturalist::apis::Error<inaturalist::apis::observations_api::ObservationsGetError>,
> {
    let params = inaturalist::apis::observations_api::ObservationsGetParams {
        swlat: Some(rect.min().y),
        swlng: Some(rect.min().x),
        nelat: Some(rect.max().y),
        nelng: Some(rect.max().x),
        quality_grade: Some(String::from("research")),
        taxon_id: Some(vec![PLANTAE_ID.to_string()]),
        per_page: Some(200.to_string()),
        // native: Some(true),
        ..Default::default()
    };

    inaturalist::apis::observations_api::observations_get(&INATURALIST_REQUEST_CONFIG, params).await
}

fn grid_iter(
    rect: geo_types::Rect<f64>,
    divisions: u32,
) -> impl Iterator<Item = geo_types::Rect<f64>> {
    let grid_width = rect.width() / (divisions as f64);

    (0..(divisions * divisions)).map(move |n| {
        let x_offset = n % divisions;
        let y_offset = n / divisions;

        let sw_x = rect.min().x + (grid_width * (x_offset as f64));
        let sw_y = rect.min().y + (grid_width * (y_offset as f64));

        geo_types::Rect::new(
            geo_types::coord! { x: sw_x, y: sw_y, },
            geo_types::coord! { x: sw_x + grid_width, y: sw_y + grid_width, },
        )
    })
}
