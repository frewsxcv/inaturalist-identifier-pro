use std::collections;

type Url = String;

#[derive(Default)]
pub struct ImageStore {
    pub hash_map: collections::HashMap<i32, (Url, egui_extras::RetainedImage)>,
}

impl ImageStore {
    pub fn insert(&mut self, observation_id: i32, url: String, image: egui_extras::RetainedImage) {
        let _ = self.hash_map.insert(observation_id, (url, image));
    }

    pub fn load(&self, observation_id: i32) -> Option<&(String, egui_extras::RetainedImage)> {
        self.hash_map.get(&observation_id)
    }
}
