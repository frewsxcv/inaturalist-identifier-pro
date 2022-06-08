use std::collections;

#[derive(Default)]
pub(crate) struct ImageStore {
    hash_map: collections::HashMap<i32, egui_extras::RetainedImage>,
}

impl ImageStore {
    pub fn insert(&mut self, observation_id: i32, image: egui_extras::RetainedImage) {
	let _ = self.hash_map.insert(observation_id, image);
    }

    pub fn load(&self, observation_id: i32) -> Option<&egui_extras::RetainedImage> {
        self.hash_map.get(&observation_id)
    }
}
