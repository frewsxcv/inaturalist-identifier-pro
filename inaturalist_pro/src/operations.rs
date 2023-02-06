use inaturalist::models::Observation;
use std::{collections, fs, mem};

pub trait Operation {
    async fn visit_observation(&mut self, _observation: &crate::Observation) {}
    // async fn visit_geohash_observations(
    //     &mut self,
    //     _geohash: crate::Geohash,
    //     _observations: &crate::Observations,
    // ) {
    // }
    fn finish(&mut self) {}
}

#[derive(Default)]
pub struct NoOp(pub Vec<Observation>);

impl Operation for NoOp {
    async fn visit_observation(&mut self, observation: &crate::Observation) {
        self.0.push(observation.clone());
    }
}

#[derive(Default)]
pub struct TopImageScore(pub Vec<Observation>);

impl Operation for TopImageScore {
    async fn visit_observation(&mut self, observation: &crate::Observation) {
        let results =
            inaturalist_fetch::fetch_computer_vision_observation_scores(observation).await;
        let url = observation.uri.clone().unwrap_or_default();
        let score = results.results[0].vision_score;
        println!("{url} - score: {score}");
    }
}

#[derive(Default)]
pub struct PrintPlantae(pub Vec<Observation>);

impl Operation for PrintPlantae {
    async fn visit_observation(&mut self, observation: &crate::Observation) {
        if let Some(taxon) = &observation.taxon {
            if taxon.rank == Some("kingdom".to_string()) && observation.captive == Some(false) {
                self.0.push(observation.clone());
            }
        }
    }
}

#[derive(Default)]
pub struct PrintAngiospermae(pub Vec<Observation>);

impl Operation for PrintAngiospermae {
    async fn visit_observation(&mut self, observation: &crate::Observation) {
        if let Some(taxon) = &observation.taxon {
            if taxon.id == Some(47125) && observation.captive == Some(false) {
                self.0.push(observation.clone());
            }
        }
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
