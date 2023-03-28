use std::collections;

use inaturalist::models::{ObservationTaxon, ShowTaxon};

pub type TaxaId = i32;

#[derive(Default)]
pub struct TaxaStore(pub collections::HashMap<TaxaId, TaxaValue>);

pub enum TaxaValue {
    Loading,
    Loaded(Taxon),
}

impl From<&ShowTaxon> for TaxaValue {
    fn from(value: &ShowTaxon) -> Self {
        TaxaValue::Loaded(value.into())
    }
}

#[derive(Debug)]
pub struct Taxon {
    pub id: TaxaId,
    pub name: String,
}

impl From<&ShowTaxon> for Taxon {
    fn from(value: &ShowTaxon) -> Self {
        Taxon {
            id: value.id.unwrap(),
            name: value
                .preferred_common_name
                .clone()
                .unwrap_or_else(|| value.name.clone().unwrap()),
        }
    }
}

impl From<&ObservationTaxon> for Taxon {
    fn from(value: &ObservationTaxon) -> Self {
        Taxon {
            id: value.id.unwrap(),
            name: value
                .preferred_common_name
                .clone()
                .unwrap_or_else(|| value.name.clone().unwrap()),
        }
    }
}
