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

type ObservationId = i32;

pub(crate) struct App {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub results: Vec<QueryResult>,
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
    pub taxa_store: TaxaStore,
    pub current_observation_id: Option<ObservationId>,
}

pub struct QueryResult {
    observation: Observation,
    scores: Option<Vec<inaturalist_fetch::ComputerVisionObservationScore>>,
    taxon_tree: crate::taxon_tree::TaxonTree,
}

impl App {
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
                crate::AppMessage::ObservationLoaded(observation) => {
                    if self.current_observation_id.is_none() {
                        self.current_observation_id = observation.id;
                    }

                    self.results.push(QueryResult {
                        observation: *observation.clone(),
                        scores: None,
                        taxon_tree: Default::default(),
                    });

                    self.load_image_in_background_thread(observation);
                }
                crate::AppMessage::ComputerVisionScoreLoaded(observation_id, scores) => {
                    let Some(observation_index) = self.find_index_for_observation_id(observation_id) else {
                        // TODO: Log error here
                        return;
                    };
                    self.build_taxon_tree_in_background_thread(
                        &self.results[observation_index].observation,
                        scores.clone(),
                    );
                    self.results[observation_index].scores = Some(scores);
                    self.sort_results();
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
                crate::AppMessage::SkipCurrentObservation => {
                    let Some(current_observation_index) = self.find_index_for_current_observation() else {
                        return
                    };
                    self.results.remove(current_observation_index);
                    self.select_new_observation();
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
                let Some(current_observation_index) = self.find_index_for_current_observation() else {
                    return
                };
                let query_result = &self.results[current_observation_index];
                if ui.button("Skip observation").clicked() {
                    self.tx_app_message.send(crate::AppMessage::SkipCurrentObservation).unwrap();
                }
                ui.horizontal(|ui| {
                    if let Some((url, image)) = self
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
                                let response = image.show_size(ui, image_size);
                                if response.clicked() {
                                    tracing::info!("Clicked the image");
                                    ui.ctx().output_mut(|o| {
                                        o.open_url = Some(egui::output::OpenUrl {
                                            url: url.into(),
                                            new_tab: true,
                                        });
                                    });
                                }
                                response
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
                                    self.tx_app_message.send(crate::AppMessage::SkipCurrentObservation).unwrap();
                                }
                            }
                        });
                    } else {
                        ui.spinner();
                    }
                });
                ui.separator();
            });
        });
    }
}

impl App {
    fn select_new_observation(&mut self) {
        self.current_observation_id = self.results.first().map(|o| o.observation.id.unwrap());
    }

    fn find_index_for_current_observation(&self) -> Option<usize> {
        self.current_observation_id
            .and_then(|id| self.find_index_for_observation_id(id))
    }

    fn find_index_for_observation_id(&self, observation_id: ObservationId) -> Option<usize> {
        self.results
            .iter()
            .enumerate()
            .find(|(_, result)| result.observation.id.unwrap() == observation_id)
            .map(|(i, _)| i)
    }

    fn sort_results(&mut self) {
        self.results.sort_unstable_by(|a, b| {
            let score_a = a
                .scores
                .as_ref()
                .map_or(0., |scores| scores[0].combined_score);
            let score_b = b
                .scores
                .as_ref()
                .map_or(0., |scores| scores[0].combined_score);
            score_a.partial_cmp(&score_b).unwrap().reverse()
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
