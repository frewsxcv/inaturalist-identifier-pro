use actix::{Arbiter, SystemService, Actor};
use inaturalist::models::Observation;
use std::sync;

use crate::{
    image_store_actor::{ImageStoreActor, LoadImageMessage},
    taxon_tree::TaxonTreeNode,
    taxon_tree_builder_actor::{TaxonTreeBuilderActor, BuildTaxonTreeMessage},
};

pub(crate) struct App {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub total_geohashes: usize,
    pub results: Vec<QueryResult>,
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
}

pub struct QueryResult {
    observation: Observation,
    scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    taxon_tree: crate::taxon_tree::TaxonTree,
}

impl App {
    fn handle_new_result(
        &mut self,
        observation: Box<Observation>,
        scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ) {
        self.results.push(QueryResult {
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

        TaxonTreeBuilderActor::from_registry()
            .try_send(BuildTaxonTreeMessage {
                observation_id: observation.id.unwrap(),
                scores,
            })
            .unwrap();

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
        ImageStoreActor::from_registry()
            .try_send(LoadImageMessage { observation })
            .unwrap();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(app_message) = self.rx_app_message.try_recv() {
            match app_message {
                crate::AppMessage::Progress => {
                    self.loaded_geohashes += 1;
                }
                crate::AppMessage::Result((observation, scores)) => {
                    self.handle_new_result(observation, scores)
                }
                crate::AppMessage::TaxonTree {
                    observation_id,
                    taxon_tree,
                } => {
                    for n in &mut self.results {
                        if n.observation.id == Some(observation_id) {
                            n.taxon_tree = taxon_tree;
                            break;
                        }
                    }
                }
            }
        }

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
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Results");
                for foo in &self.results {
                    ui.horizontal(|ui| {
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

                            ui.vertical(|ui| {
                                ui.hyperlink(foo.observation.uri.as_ref().unwrap());
                                for score in &foo.scores {
                                    ui.label(format!(
                                        "Guess: {}",
                                        score.taxon.name.as_ref().unwrap()
                                    ));
                                    ui.label(format!("Score: {}", score.combined_score));
                                }
                                ui.heading("Taxon tree");
                                if foo.taxon_tree.0.is_empty() {
                                    ui.spinner();
                                } else {
                                    for (_, v) in foo.taxon_tree.0.iter() {
                                        ui.add(TaxonTreeWidget {
                                            observation: foo.observation.clone(),
                                            root_node: v.clone(),
                                        });
                                    }
                                }
                            });
                        } else {
                            ui.spinner();
                        }
                    });
                    ui.separator();
                }
            });
        });
    }
}

struct TaxonTreeWidget {
    observation: Observation,
    root_node: crate::taxon_tree::TaxonTreeNode,
}

impl egui::Widget for TaxonTreeWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let collapsing_header_id = format!(
            "{}-{}",
            self.observation.id.unwrap(),
            self.root_node.taxon.id.unwrap(),
        );

        egui::CollapsingHeader::new(self.root_node.taxon.preferred_common_name.as_ref().unwrap())
            .id_source(collapsing_header_id)
            .default_open(true)
            .show(ui, |ui| {
                for child in self.root_node.children.0.values() {
                    ui.add(TaxonTreeWidget {
                        observation: self.observation.clone(),
                        root_node: child.clone(),
                    });
                }
            })
            .header_response
        // .bodyreturned
        // .unwrap_or(Action::Keep)
    }
}
