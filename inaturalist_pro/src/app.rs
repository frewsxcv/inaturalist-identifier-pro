use actix::SystemService;
use egui::Color32;
use inaturalist::models::Observation;
use std::sync;

use crate::{
    image_store_actor::{ImageStoreActor, LoadImageMessage},
    taxa_store::{TaxaValue, TaxaStore},
    taxon_tree_builder_actor::{BuildTaxonTreeMessage, TaxonTreeBuilderActor},
};

pub(crate) struct App {
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub results: Vec<QueryResult>,
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
    pub taxa_store: TaxaStore,
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

        self.results.sort_unstable_by(|a, b| {
            a.scores[0]
                .combined_score
                .partial_cmp(&b.scores[0].combined_score)
                .unwrap()
                .reverse()
        });

        self.build_taxon_tree_in_background_thread(&observation, scores);
        self.load_image_in_background_thread(observation);
    }

    fn build_taxon_tree_in_background_thread(
        &self,
        observation: &Observation,
        scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ) {
        TaxonTreeBuilderActor::from_registry()
            .try_send(BuildTaxonTreeMessage {
                observation_id: observation.id.unwrap(),
                scores,
            })
            .unwrap();
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
                crate::AppMessage::TaxonLoaded(taxon) => {
                    self.taxa_store.0.insert(taxon.id.unwrap(), (&*taxon).into());
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
                for query_result in &self.results {
                    ui.horizontal(|ui| {
                        if let Some(image) = self
                            .image_store
                            .read()
                            .unwrap()
                            .load(query_result.observation.id.unwrap())
                        {
                            ui.add_sized(image.size_vec2(), |ui: &mut egui::Ui| {
                                if ui.max_rect().intersects(rect) {
                                    image.show(ui)
                                } else {
                                    ui.spinner()
                                }
                            });

                            ui.vertical(|ui| {
                                ui.hyperlink(query_result.observation.uri.as_ref().unwrap());
                                ui.heading("Taxon tree");
                                if query_result.taxon_tree.0.is_empty() {
                                    ui.spinner();
                                } else {
                                    for node in query_result.taxon_tree.0.iter() {
                                        ui.add(TaxonTreeWidget {
                                            observation: &query_result.observation,
                                            root_node: node,
                                            taxa_store: &self.taxa_store,
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

struct TaxonTreeWidget<'a> {
    observation: &'a Observation,
    root_node: &'a crate::taxon_tree::TaxonTreeNode,
    taxa_store: &'a TaxaStore,
}

impl<'a> egui::Widget for TaxonTreeWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let collapsing_header_id = format!(
            "{}-{}",
            self.observation.id.unwrap(),
            self.root_node.taxon_id,
        );

        let title = match self.taxa_store.0.get(&self.root_node.taxon_id) {
            Some(TaxaValue::Loaded(taxon)) => {
                format!("{} ({})", self.root_node.taxon_id, taxon.name,)
            }
            Some(TaxaValue::Loading) => "Loading...".to_string(),
            None => "<Unknown>".to_string(),
        };

        egui::CollapsingHeader::new(title)
            .id_source(collapsing_header_id)
            .default_open(true)
            .show(ui, |ui| {
                let color = if self.root_node.score > 75. {
                    Color32::BLUE
                } else if self.root_node.score > 50. {
                    Color32::GREEN
                } else if self.root_node.score > 25. {
                    Color32::YELLOW
                } else if self.root_node.score > 0. {
                    Color32::RED
                } else {
                    Color32::GRAY
                };
                ui.colored_label(color, format!("Score: {}", self.root_node.score));
                for node in self.root_node.children.0.iter() {
                    ui.add(TaxonTreeWidget {
                        observation: self.observation,
                        root_node: node,
                        taxa_store: self.taxa_store,
                    });
                }
            })
            .header_response
    }
}
