use crate::Rect;
use std::sync::OnceLock;

#[allow(dead_code)]
static INDIAN_LAKE_CELL: OnceLock<Rect> = OnceLock::new();
#[allow(dead_code)]
pub fn indian_lake() -> &'static Rect {
    INDIAN_LAKE_CELL.get_or_init(|| {
        geo::Rect::new(
            geo::coord! {
                x: ordered_float::OrderedFloat(-74.3812336),
                y: ordered_float::OrderedFloat(43.6464133),
            },
            geo::coord! {
                x: ordered_float::OrderedFloat(-74.270485),
                y: ordered_float::OrderedFloat(43.756746),
            },
        )
    })
}

#[allow(dead_code)]
static HARRIMAN_STATE_PARK_CELL: OnceLock<Rect> = OnceLock::new();
#[allow(dead_code)]
pub fn harriman_state_park() -> &'static Rect {
    HARRIMAN_STATE_PARK_CELL.get_or_init(|| {
        geo::Rect::new(
            geo::coord! {
                x: ordered_float::OrderedFloat(-74.26345825195312),
                y: ordered_float::OrderedFloat(41.101086483800515),
            },
            geo::coord! {
                x: ordered_float::OrderedFloat(-73.948873),
                y: ordered_float::OrderedFloat(41.34124700339191)
            },
        )
    })
}

#[allow(dead_code)]
static BROOKLYN_CELL: OnceLock<Rect> = OnceLock::new();
#[allow(dead_code)]
pub fn brooklyn() -> &'static Rect {
    BROOKLYN_CELL.get_or_init(|| {
        geo::Rect::new(
            geo::coord! {
                x: ordered_float::OrderedFloat(-74.046000f64),
                y: ordered_float::OrderedFloat(40.567),
            },
            geo::coord! {
                x: ordered_float::OrderedFloat(-73.9389741f64),
                y: ordered_float::OrderedFloat(40.6942535f64),
            },
        )
    })
}

#[allow(dead_code)]
static PROSPECT_PARK_CELL: OnceLock<Rect> = OnceLock::new();
#[allow(dead_code)]
pub fn prospect_park() -> &'static Rect {
    PROSPECT_PARK_CELL.get_or_init(|| {
        geo::Rect::new(
            geo::coord! {
                x: ordered_float::OrderedFloat(-73.979336),
                y: ordered_float::OrderedFloat(40.650289),
            },
            geo::coord! {
                x: ordered_float::OrderedFloat(-73.9722377),
                y: ordered_float::OrderedFloat(40.6594511),
            },
        )
    })
}

static NYC_CELL: OnceLock<Rect> = OnceLock::new();
pub fn nyc() -> &'static Rect {
    NYC_CELL.get_or_init(|| {
        geo::Rect::new(
            geo::coord! {
                x: ordered_float::OrderedFloat(-74.258019),
                y: ordered_float::OrderedFloat(40.490742)
            },
            geo::coord! {
                x: ordered_float::OrderedFloat(-73.555615),
                y: ordered_float::OrderedFloat(41.017433)
            },
        )
    })
}
