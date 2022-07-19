use crate::Rect;

lazy_static::lazy_static! {
    pub static ref HARRIMAN_STATE_PARK: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-74.26345825195312),
            y: ordered_float::OrderedFloat(41.101086483800515),
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.948873),
            y: ordered_float::OrderedFloat(41.34124700339191)
        },
    );

    pub static ref BROOKLYN: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-74.046000f64),
            y: ordered_float::OrderedFloat(40.567),
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.9389741f64),
            y: ordered_float::OrderedFloat(40.6942535f64),
        },
    );

    pub static ref PROSPECT_PARK: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.979336),
            y: ordered_float::OrderedFloat(40.650289),
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.9722377),
            y: ordered_float::OrderedFloat(40.6594511),
        },
    );

    pub static ref NYC: Rect = geo::Rect::new(
        geo::coord! {
            x: ordered_float::OrderedFloat(-74.258019),
            y: ordered_float::OrderedFloat(40.490742)
        },
        geo::coord! {
            x: ordered_float::OrderedFloat(-73.555615),
            y: ordered_float::OrderedFloat(41.017433)
        },
    );
}
