use crate::{
    actors::identify_actor::{IdentifyActor, IdentifyMessage},
    taxa_store::TaxaStore,
};

use actix::SystemService;

use egui::{Sense, Vec2};
use inaturalist::models::Observation;

const MAX_SCORE: f64 = 100.;

pub struct TaxonTreeWidget<'a> {
    pub observation: &'a Observation,
    pub root_node: &'a crate::taxon_tree::TaxonTreeNode,
    pub taxa_store: &'a TaxaStore,
    pub identified: &'a mut bool,
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
                    ui.horizontal(|ui| {
                        // Score square
                        let score_color = colorous::COOL
                            .eval_continuous(self.root_node.score.round() as f64 / MAX_SCORE);
                        let rect_size = Vec2::new(ui.available_height(), ui.available_height());
                        let (rect, response) = ui.allocate_exact_size(rect_size, Sense::hover());
                        response.on_hover_text(format!(
                            "Score: {} / {}",
                            self.root_node.score.round(),
                            MAX_SCORE
                        ));
                        let shape = egui::Shape::rect_filled(
                            rect,
                            egui::CornerRadius::default(),
                            egui::Color32::from_rgb(score_color.r, score_color.g, score_color.b),
                        );
                        ui.painter().add(shape);

                        ui.label(&taxon.name);

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

                        ui.hyperlink_to(
                            "🌎",
                            format!(
                                "https://www.inaturalist.org/taxa/{}",
                                self.root_node.taxon_id
                            ),
                        );
                    });
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
