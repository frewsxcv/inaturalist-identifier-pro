use inaturalist::models::Observation;
use std::error;

use crate::AppMessage;

// const PLANTAE_ID: u32 = 47126;
// const INSECTA_ID: u32 = 47158;
const DIPTERA_ID: u32 = 47822;

pub trait Operation {
    fn request() -> inaturalist::apis::observations_api::ObservationsGetParams {
        inaturalist::apis::observations_api::ObservationsGetParams {
            acc: None,
            captive: None,
            endemic: None,
            geo: None,
            id_please: None,
            identified: None,
            introduced: None,
            mappable: None,
            native: None,
            out_of_range: None,
            pcid: None,
            photos: None,
            popular: None,
            sounds: None,
            taxon_is_active: None,
            threatened: None,
            verifiable: None,
            licensed: None,
            photo_licensed: None,
            expected_nearby: None,
            id: None,
            not_id: None,
            license: None,
            ofv_datatype: None,
            photo_license: None,
            place_id: None,
            project_id: None,
            rank: None,
            site_id: None,
            sound_license: None,
            taxon_id: None,
            without_taxon_id: None,
            taxon_name: None,
            user_id: None,
            user_login: None,
            ident_user_id: None,
            hour: None,
            day: None,
            month: None,
            year: None,
            created_day: None,
            created_month: None,
            created_year: None,
            term_id: None,
            term_value_id: None,
            without_term_id: None,
            without_term_value_id: None,
            term_id_or_unknown: None,
            annotation_user_id: None,
            acc_above: None,
            acc_below: None,
            acc_below_or_unknown: None,
            d1: None,
            d2: None,
            created_d1: None,
            created_d2: None,
            created_on: None,
            observed_on: None,
            unobserved_by_user_id: None,
            apply_project_rules_for: None,
            cs: None,
            csa: None,
            csi: None,
            geoprivacy: None,
            taxon_geoprivacy: None,
            obscuration: None,
            hrank: None,
            lrank: None,
            iconic_taxa: None,
            id_above: None,
            id_below: None,
            identifications: None,
            lat: None,
            lng: None,
            radius: None,
            nelat: None,
            nelng: None,
            swlat: None,
            swlng: None,
            list_id: None,
            not_in_project: None,
            not_matching_project_rules_for: None,
            observation_accuracy_experiment_id: None,
            q: None,
            search_on: None,
            quality_grade: None,
            updated_since: None,
            viewer_id: None,
            reviewed: None,
            locale: None,
            preferred_place_id: None,
            ttl: None,
            page: None,
            per_page: None,
            order: None,
            order_by: None,
            only_id: None,
        }
    }

    fn visit_observation(
        &mut self,
        _observation: crate::Observation,
        _tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
        _api_token: &str,
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
        _api_token: &str,
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
            order_by: Some("created_at".to_string()),
            order: Some("asc".to_string()),
            acc: None,
            captive: None,
            endemic: None,
            geo: None,
            id_please: None,
            identified: None,
            introduced: None,
            mappable: None,
            native: None,
            out_of_range: None,
            pcid: None,
            photos: None,
            popular: None,
            sounds: None,
            taxon_is_active: None,
            threatened: None,
            verifiable: None,
            licensed: None,
            photo_licensed: None,
            expected_nearby: None,
            id: None,
            not_id: None,
            license: None,
            ofv_datatype: None,
            photo_license: None,
            place_id: None,
            project_id: None,
            rank: None,
            site_id: None,
            sound_license: None,
            without_taxon_id: None,
            taxon_name: None,
            user_id: None,
            user_login: None,
            ident_user_id: None,
            hour: None,
            day: None,
            month: None,
            year: None,
            created_day: None,
            created_month: None,
            created_year: None,
            term_id: None,
            term_value_id: None,
            without_term_id: None,
            without_term_value_id: None,
            term_id_or_unknown: None,
            annotation_user_id: None,
            acc_above: None,
            acc_below: None,
            acc_below_or_unknown: None,
            d1: None,
            d2: None,
            created_d1: None,
            created_d2: None,
            created_on: None,
            observed_on: None,
            unobserved_by_user_id: None,
            apply_project_rules_for: None,
            cs: None,
            csa: None,
            csi: None,
            geoprivacy: None,
            taxon_geoprivacy: None,
            obscuration: None,
            hrank: None,
            iconic_taxa: None,
            id_above: None,
            id_below: None,
            identifications: None,
            lat: None,
            lng: None,
            radius: None,
            nelat: None,
            nelng: None,
            swlat: None,
            swlng: None,
            list_id: None,
            not_in_project: None,
            not_matching_project_rules_for: None,
            observation_accuracy_experiment_id: None,
            q: None,
            search_on: None,
            updated_since: None,
            locale: None,
            preferred_place_id: None,
            ttl: None,
            page: None,
            per_page: None,
            only_id: None,
        }
    }

    fn visit_observation(
        &mut self,
        observation: crate::Observation,
        tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
        api_token: &str,
    ) -> Result<(), Box<dyn error::Error>> {
        tracing::info!("VISIT OBSERVATION");
        let api_token = api_token.to_string();
        actix::spawn(async move {
            let observation_id = observation.id.unwrap();
            tx_app_message
                .send(AppMessage::ObservationLoaded(Box::new(observation.clone())))
                .unwrap();
            let results = inaturalist_fetch::fetch_computer_vision_observation_scores(
                &observation,
                &api_token,
            )
            .await;
            tx_app_message
                .send(AppMessage::ComputerVisionScoreLoaded(
                    observation_id,
                    results.results,
                ))
                .unwrap();
        });
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
        _api_token: &str,
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
        _api_token: &str,
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

/*
fn observations_species_count(observations: &[Observation]) -> usize {
    // TODO this should actually be a ratio?
    observations
        .iter()
        .filter_map(|observation| observation.taxon.as_ref())
        .map(|taxon| taxon.id)
        .collect::<collections::HashSet<_>>()
        .len()
}
*/

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

// type GeohashTopObservers = collections::HashMap<String, usize>;

/*
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
*/
