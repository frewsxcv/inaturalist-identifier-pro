use inaturalist::models::Observation;
use std::{collections, fs, mem};

pub trait Operation {
    fn visit_observation(&mut self, _observation: &crate::Observation) {}
    fn visit_geohash_observations(
        &mut self,
        _geohash: &crate::Geohash,
        _observations: &crate::Observations,
    ) {
    }
    fn finish(&mut self) {}
}

#[derive(Default)]
pub struct PrintPlantae(Vec<String>);

impl Operation for PrintPlantae {
    fn visit_observation(&mut self, observation: &crate::Observation) {
        if let Some(taxon) = &observation.taxon {
            if taxon.rank == Some("kingdom".to_string()) && observation.captive == Some(false) {
                self.0.push(observation.uri.as_ref().unwrap().into());
            }
        }
    }

    fn finish(&mut self) {
        let native_options = eframe::NativeOptions::default();
        let urls = self.0.clone();
        eframe::run_native(
            "eframe template",
            native_options,
            Box::new(|_| Box::new(crate::app::TemplateApp::new(urls))),
        );
    }
}

pub struct GeoJsonUniqueSpecies {
    geojson_features: Vec<geojson::Feature>,
}

impl Operation for GeoJsonUniqueSpecies {
    fn visit_geohash_observations(
        &mut self,
        geohash: &crate::Geohash,
        observations: &crate::Observations,
    ) {
        let mut geojson_feature = geohash.to_geojson_feature();
        let species_count = observations_species_count(observations);
        if let Some(properties) = &mut geojson_feature.properties {
            properties.insert("species count".into(), species_count.into());
        }
        self.geojson_features.push(geojson_feature);
    }

    fn finish(&mut self) {
        let geojson_feature_collection = geojson::FeatureCollection {
            features: mem::take(&mut self.geojson_features),
            bbox: None,
            foreign_members: None,
        };

        fs::write(
            "/Users/coreyf/tmp/output.geojson",
            geojson_feature_collection.to_string(),
        )
        .unwrap();
    }
}

fn observations_species_count(observations: &[Observation]) -> usize {
    // TODO this should actually be a ratio?
    observations
        .iter()
        .filter_map(|observation| observation.taxon.as_ref())
        .map(|taxon| taxon.id)
        .collect::<collections::HashSet<_>>()
        .len()
}
