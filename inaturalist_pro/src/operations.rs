use inaturalist::models::Observation;
use std::{collections, error};

use crate::AppMessage;

// const PLANTAE_ID: u32 = 47126;
// const INSECTA_ID: u32 = 47158;
const DIPTERA_ID: u32 = 47822;

pub trait Operation {
    fn request() -> inaturalist::apis::observations_api::ObservationsGetParams {
        inaturalist::apis::observations_api::ObservationsGetParams::default()
    }

    fn visit_observation(
        &mut self,
        _observation: crate::Observation,
        _tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    // async fn visit_geohash_observations(
    //     &mut self,
    //     _geohash: crate::Geohash,
    //     _observations: &crate::Observations,
    // ) {
    // }
    fn finish(&mut self, _tx_app_message: tokio::sync::mpsc::Sender<crate::AppMessage>) {}
}

pub struct NoOp(pub Vec<Observation>);

impl Operation for NoOp {
    fn visit_observation(
        &mut self,
        observation: crate::Observation,
        _tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    ) -> Result<(), Box<dyn error::Error>> {
        self.0.push(observation);
        Ok(())
    }
}

#[derive(Default)]
pub struct TopImageScore(pub Vec<Observation>);

impl Operation for TopImageScore {
    fn request() -> inaturalist::apis::observations_api::ObservationsGetParams {
        inaturalist::apis::observations_api::ObservationsGetParams {
            viewer_id: Some("3191422".into()),
            reviewed: Some(false),
            quality_grade: Some(String::from("needs_id")),
            taxon_id: Some(vec![DIPTERA_ID.to_string()]),
            lrank: Some("suborder".to_string()),
            ..Default::default()
        }
    }

    fn visit_observation(
        &mut self,
        observation: crate::Observation,
        tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    ) -> Result<(), Box<dyn error::Error>> {
        let results = futures::executor::block_on(
            inaturalist_fetch::fetch_computer_vision_observation_scores(&observation),
        );
        let _url = observation.uri.clone().unwrap_or_default();
        tx_app_message.send(AppMessage::Result((Box::new(observation), results.results)))?;
        Ok(())
    }
}

#[derive(Default)]
pub struct PrintPlantae(pub Vec<Observation>);

impl Operation for PrintPlantae {
    fn visit_observation(
        &mut self,
        observation: crate::Observation,
        _tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    ) -> Result<(), Box<dyn error::Error>> {
        if let Some(taxon) = &observation.taxon {
            if taxon.rank == Some("kingdom".to_string()) && observation.captive == Some(false) {
                self.0.push(observation.clone());
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct PrintAngiospermae(pub Vec<Observation>);

impl Operation for PrintAngiospermae {
    fn visit_observation(
        &mut self,
        observation: crate::Observation,
        _tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    ) -> Result<(), Box<dyn error::Error>> {
        if let Some(taxon) = &observation.taxon {
            if taxon.id == Some(47125) && observation.captive == Some(false) {
                self.0.push(observation.clone());
            }
        }
        Ok(())
    }
}
// pub struct GeoJsonUniqueSpecies {
//     geojson_features: Vec<geojson::Feature>,
// }

// impl Operation for GeoJsonUniqueSpecies {
//     async fn visit_geohash_observations(
//         &mut self,
//         geohash: crate::Geohash,
//         observations: &crate::Observations,
//     ) {
//         let mut geojson_feature = geohash.to_geojson_feature();
//         let species_count = observations_species_count(observations);
//         if let Some(properties) = &mut geojson_feature.properties {
//             properties.insert("species count".into(), species_count.into());
//         }
//         self.geojson_features.push(geojson_feature);
//     }

//     fn finish(&mut self) {
//         let geojson_feature_collection = geojson::FeatureCollection {
//             features: mem::take(&mut self.geojson_features),
//             bbox: None,
//             foreign_members: None,
//         };

//         fs::write(
//             "/Users/coreyf/tmp/output.geojson",
//             geojson_feature_collection.to_string(),
//         )
//         .unwrap();
//     }
// }

fn observations_species_count(observations: &[Observation]) -> usize {
    // TODO this should actually be a ratio?
    observations
        .iter()
        .filter_map(|observation| observation.taxon.as_ref())
        .map(|taxon| taxon.id)
        .collect::<collections::HashSet<_>>()
        .len()
}

// #[derive(Default)]
// pub struct TopObservationsPerTile {
//     observations: collections::HashMap<crate::Geohash, GeohashTopObservers>,
// }

// impl Operation for TopObservationsPerTile {
//     async fn visit_geohash_observations(
//         &mut self,
//         geohash: crate::Geohash,
//         observations: &crate::Observations,
//     ) {
//         self.observations
//             .insert(geohash, observations_top_observers(observations));
//     }

//     fn finish(&mut self) {
//         println!("{:?}", self.observations);
//     }
// }

type GeohashTopObservers = collections::HashMap<String, usize>;

fn observations_top_observers(observations: &[Observation]) -> GeohashTopObservers {
    let mut map = collections::HashMap::new();
    for observation in observations.iter() {
        *map.entry(
            observation
                .user
                .as_ref()
                .expect("could not fetch user")
                .login
                .as_ref()
                .expect("could not fetch user's login")
                .clone(),
        )
        .or_default() += 1;
    }
    map
}
