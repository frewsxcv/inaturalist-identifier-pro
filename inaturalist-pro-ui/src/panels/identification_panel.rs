use crate::widgets::taxon_tree::TaxonTreeWidget;
use inaturalist_pro_core::{AppMessage, QueryResult, TaxaStore};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct IdentificationPanel;

impl IdentificationPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        query_result: Option<&QueryResult>,
        taxa_store: &TaxaStore,
        tx_app_message: &UnboundedSender<AppMessage>,
    ) {
        egui::SidePanel::right("identification_panel").show(ctx, |ui| {
            ui.heading("Identification Panel");
            let Some(query_result) = query_result else {
                // TODO: Show a message here?
                return;
            };

            if query_result.taxon_tree.nodes.is_empty() {
                ui.spinner();
            } else {
                let mut identified = false;
                for node in query_result.taxon_tree.nodes.values() {
                    ui.add(TaxonTreeWidget {
                        observation: &query_result.observation,
                        root_node: node,
                        taxa_store,
                        identified: &mut identified,
                    });
                }
                if identified {
                    tx_app_message
                        .send(inaturalist_pro_core::AppMessage::SkipCurrentObservation)
                        .unwrap();
                }
            }
        });
    }
}
