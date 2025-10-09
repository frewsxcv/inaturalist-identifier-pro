pub mod geohash_ext;
pub mod geohash_observations;
pub mod places;

pub use geohash_ext::{Geohash, GeohashGrid};
pub use geohash_observations::GeohashObservations;

pub type Rect = geo::Rect<ordered_float::OrderedFloat<f64>>;
