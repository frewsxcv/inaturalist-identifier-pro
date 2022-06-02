use geo::Convert;

#[derive(Clone)]
pub struct Geohash {
    pub string: String,
    pub bounding_rect: crate::Rect,
}

impl Geohash {
    pub fn to_geojson_feature(&self) -> geojson::Feature {
        let mut properties = geojson::JsonObject::new();
        properties.insert(
            String::from("geohash"),
            geojson::JsonValue::from(&*self.string),
        );
        let value: geojson::Value =
            geojson::Value::try_from(&self.bounding_rect.to_polygon()).unwrap();
        geojson::Feature {
            properties: Some(properties),
            geometry: Some(value.into()),
            ..Default::default()
        }
    }
}

pub struct GeohashGrid(pub Vec<Geohash>);

impl GeohashGrid {
    pub fn from_rect(rect: crate::Rect, geohash_len: usize) -> Self {
        GeohashGrid(
            geohashes_within_rect(rect, geohash_len)
                .map(|geohash_string| {
                    let bbox = geohash::decode_bbox(&geohash_string).unwrap();
                    Geohash {
                        string: geohash_string,
                        bounding_rect: bbox.convert(),
                    }
                })
                .collect(),
        )
    }

    #[allow(unused)]
    pub fn to_geojson_feature_collection(&self) -> geojson::FeatureCollection {
        let mut features = vec![];
        for geohash in &self.0 {
            features.push(geohash.to_geojson_feature());
        }
        geojson::FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        }
    }
}

fn geohashes_within_rect<T>(rect: geo::Rect<T>, len: usize) -> Iter
where
    T: geo::CoordNum,
    f64: From<T>,
{
    Iter {
        rect: rect.convert(),
        len,
        last: None,
    }
}

struct Iter {
    rect: geo::Rect<f64>,
    len: usize,
    last: Option<String>,
}

impl Iter {
    fn first(&self) -> String {
        let min = geo::coord! {
            x: self.rect.min().x,
            y: self.rect.min().y
        };
        geohash::encode(min, self.len).unwrap()
    }
}

impl Iterator for Iter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.last = match &self.last {
            Some(last) => {
                let geohash_rect = geohash::decode_bbox(last).unwrap();
                // If:
                //
                // - The last geohash has not exceeded rect's x max, then find the east neighbor
                if geohash_rect.max().x < self.rect.max().x {
                    Some(geohash::neighbor(last, geohash::Direction::E).unwrap())
                // If:
                //
                // - The last geohash has exceeded rect's x max, and...
                // - The last geohash has not exceeded rect's y max, then start the next North row, starting from the West
                } else if geohash_rect.max().y < self.rect.max().y {
                    let min = geo::coord! {
                        x: self.rect.min().x,
                        y: geohash_rect.max().y + f64::MIN_POSITIVE,
                    };
                    Some(geohash::encode(min, self.len).unwrap())
                } else {
                    return None;
                }
            }
            None => Some(self.first()),
        };
        self.last.clone()
    }
}
