use actix::SystemService;
use egui::{Sense, Vec2};
use inaturalist::models::Observation;
use std::sync;

const MAX_SCORE: f64 = 100.;

use crate::{
    identify_actor::{IdentifyActor, IdentifyMessage},
    image_store_actor::{ImageStoreActor, LoadImageMessage},
    taxa_store::TaxaStore,
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
                    self.taxa_store
                        .0
                        .insert(taxon.id.unwrap(), (&*taxon).into());
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
            let mut to_remove = vec![];
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Results");
                for (i, query_result) in self.results.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if let Some(image) = self
                            .image_store
                            .read()
                            .unwrap()
                            .load(query_result.observation.id.unwrap())
                        {
                            const MAX_WIDTH: f32 = 500.;
                            let scale = MAX_WIDTH / (image.width() as f32);
                            let image_size =
                                egui::Vec2::new(MAX_WIDTH, image.height() as f32 * scale);
                            ui.add_sized(image_size, |ui: &mut egui::Ui| {
                                if ui.max_rect().intersects(rect) {
                                    image.show_size(ui, image_size)
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
                                    let mut identified = false;
                                    for node in query_result.taxon_tree.0.iter() {
                                        ui.add(TaxonTreeWidget {
                                            observation: &query_result.observation,
                                            root_node: node,
                                            taxa_store: &self.taxa_store,
                                            identified: &mut identified,
                                        });
                                    }
                                    if identified {
                                        to_remove.push(i);
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
            for index in to_remove {
                self.results.remove(index);
            }
        });
    }
}

struct TaxonTreeWidget<'a> {
    observation: &'a Observation,
    root_node: &'a crate::taxon_tree::TaxonTreeNode,
    taxa_store: &'a TaxaStore,
    identified: &'a mut bool,
}

impl<'a> egui::Widget for TaxonTreeWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let collapsing_header_id = format!(
            "{}-{}",
            self.observation.id.unwrap(),
            self.root_node.taxon_id,
        );

        let (response, _, _) = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            collapsing_header_id.into(),
            true,
        )
        .show_header(ui, |ui| {
            match self.taxa_store.0.get(&self.root_node.taxon_id) {
                Some(taxon) => {
                    // Score square
                    let score_color = colorous::COOL
                        .eval_continuous(self.root_node.score.round() as f64 / MAX_SCORE);
                    let _square_width = ui.max_rect().height();
                    let rect_size = Vec2::new(ui.available_height(), ui.available_height());
                    let (rect, response) = ui.allocate_exact_size(rect_size, Sense::hover());
                    response.on_hover_text(format!(
                        "Score: {} / {}",
                        self.root_node.score.round(),
                        MAX_SCORE
                    ));
                    let shape = egui::Shape::rect_filled(
                        rect,
                        egui::Rounding::default(),
                        egui::Color32::from_rgb(score_color.r, score_color.g, score_color.b),
                    );
                    ui.painter().add(shape);

                    // Identify button
                    if ui.button("✔").clicked() {
                        IdentifyActor::from_registry()
                            .try_send(IdentifyMessage {
                                observation_id: self.observation.id.unwrap(),
                                taxon_id: taxon.id,
                            })
                            .unwrap();
                        *self.identified = true;
                    }

                    ui.label(&taxon.name);

                    ui.hyperlink_to(
                        "🌎",
                        format!(
                            "https://www.inaturalist.org/taxa/{}",
                            self.root_node.taxon_id
                        ),
                    );
                }
                None => {
                    ui.spinner();
                    ui.label("Loading");
                }
            }
        })
        .body(|ui| {
            for node in self.root_node.children.0.iter() {
                ui.add(TaxonTreeWidget {
                    observation: self.observation,
                    root_node: node,
                    taxa_store: self.taxa_store,
                    identified: self.identified,
                });
            }
        });
        response
    }
}
