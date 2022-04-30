const PLANTAE_ID: u32 = 47126;

#[tokio::main]
async fn main() {
    let configuration = inaturalist::apis::configuration::Configuration {
        base_path: "https://api.inaturalist.org/v1".into(),
        ..Default::default()
    };

    let sw = geo_types::coord! { x: -74.046000, y: 40.600007 };
    let ne = geo_types::coord! { x: -73.9389741, y: 40.6942535 };

    let foo = geo_types::Rect::new(sw, ne);

    let params = inaturalist::apis::observations_api::ObservationsGetParams {
        swlat: Some(sw.y),
        swlng: Some(sw.x),
        nelat: Some(ne.y),
        nelng: Some(ne.x),
        quality_grade: Some(String::from("research")),
        taxon_id: Some(vec![PLANTAE_ID.to_string()]),
        // native: Some(true),
        ..Default::default()
    };

    let observations = inaturalist::apis::observations_api::observations_get(
        &configuration,
        params
    ).await.unwrap();

    println!("{:#?}", observations.results.len());
}
