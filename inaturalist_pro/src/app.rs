use actix::SystemService;
use inaturalist::models::Observation;
use std::sync;

use crate::{
    image_store_actor::{ImageStoreActor, LoadImageMessage},
    taxon_tree::TaxonTreeNode,
};

pub(crate) struct TemplateApp {
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub total_geohashes: usize,
    pub results: Vec<Foo>,
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
}

pub struct Foo {
    observation: Observation,
    scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    taxon_tree: crate::taxon_tree::TaxonTree,
}

impl TemplateApp {
    fn handle_new_result(
        &mut self,
        observation: Box<Observation>,
        scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ) {
        self.results.push(Foo {
            observation: *observation.clone(),
            scores: scores.clone(),
            taxon_tree: Default::default(),
        });
        // let mut taxa_ids = observation
        //     .taxon
        //     .as_ref()
        //     .unwrap()
        //     .ancestor_ids
        //     .as_ref()
        //     .unwrap()
        //     .to_owned();
        // taxa_ids.push(observation.taxon.as_ref().unwrap().id.unwrap());

        tokio::spawn(async move {
            let scores = scores.clone();
            let taxa_ids = scores
                .iter()
                .map(|n| n.taxon.id.unwrap())
                .collect::<Vec<_>>();
            let result = inaturalist_fetch::fetch_taxa(taxa_ids).await.unwrap();
            let mut hash_map = <crate::taxon_tree::TaxonTree as Default>::default();
            for result in result.results {
                let mut foo = &mut hash_map;
                for ancestor_id in result.ancestor_ids.as_ref().unwrap() {
                    // let new = crate::taxon_tree::TaxonTreeNode {
                    // children: Default::default(),
                    // };
                    // foo.0.insert(ancestor_id, Default::default());
                    println!("NEW ANCESTOR: {ancestor_id}");
                    let taxon_tree_node =
                        foo.0.entry(*ancestor_id).or_insert_with(|| TaxonTreeNode {
                            children: Default::default(),
                            score: scores
                                .iter()
                                .find(|&score| score.taxon.id == Some(*ancestor_id))
                                .map(|score| score.combined_score),
                        });
                    foo = &mut taxon_tree_node.children;
                }
            }
            // let taxon_tree = crate::taxon_tree::TaxonTree();
            println!("{:#?}", hash_map);
        });

        self.results.sort_unstable_by(|a, b| {
            a.scores[0]
                .combined_score
                .partial_cmp(&b.scores[0].combined_score)
                .unwrap()
                .reverse()
        });

        self.load_image_in_background_thread(observation);
    }

    fn load_image_in_background_thread(&self, observation: Box<Observation>) {
        tracing::info!("SENDING A MESSAGE");
        ImageStoreActor::from_registry()
            .try_send(LoadImageMessage { observation })
            .unwrap();
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(app_message) = self.rx_app_message.try_recv() {
            match app_message {
                crate::AppMessage::Progress => {
                    self.loaded_geohashes += 1;
                }
                crate::AppMessage::Result((observation, scores)) => {
                    self.handle_new_result(observation, scores)
                }
            }
        }

        // Redraw every 1 second
        let cloned_ctx = ctx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            cloned_ctx.request_repaint();
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label(format!("Loaded observations: {}", self.results.len()));
            });

            // ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     *value += 1.0;
            // }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.loaded_geohashes < self.total_geohashes {
                    ui.heading("Loading data");
                    ui.add(egui::ProgressBar::new(
                        self.loaded_geohashes as f32 / self.total_geohashes as f32,
                    ));
                }
                ui.heading("Results");
                for foo in &self.results {
                    let hyperlink = ui.hyperlink(foo.observation.uri.as_ref().unwrap());
                    // tracing::info!("meow: {:?}", (*self.image_store.read().unwrap()).hash_map.keys());
                    if let Some(image) = self
                        .image_store
                        .read()
                        .unwrap()
                        .load(foo.observation.id.unwrap())
                    {
                        ui.add_sized(image.size_vec2(), |ui: &mut egui::Ui| {
                            if ui.max_rect().intersects(rect) {
                                image.show(ui)
                            } else {
                                ui.spinner()
                            }
                        });
                        // }
                        // tracing::info!("HIIII");
                        // TODO: print tree here

                        // CollapsingHeader::new(name)
                        // .default_open(depth < 1)
                        // .show(ui, |ui| self.children_ui(ui, depth))
                        // .body_returned
                        // .unwrap_or(Action::Keep)

                        for score in &foo.scores {
                            ui.label(format!("Guess: {}", score.taxon.name.as_ref().unwrap()));
                            ui.label(format!("Score: {}", score.combined_score));
                        }
                    } else {
                        ui.spinner();
                    }
                    ui.separator();
                }
            });
        });
    }
}
