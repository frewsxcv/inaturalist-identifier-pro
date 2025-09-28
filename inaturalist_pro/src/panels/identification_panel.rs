use crate::{
    app::QueryResult, taxa_store::TaxaStore, widgets::taxon_tree::TaxonTreeWidget, AppMessage,
};
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

            if query_result.taxon_tree.0.is_empty() {
                ui.spinner();
            } else {
                let mut identified = false;
                for node in query_result.taxon_tree.0.iter() {
                    ui.add(TaxonTreeWidget {
                        observation: &query_result.observation,
                        root_node: node,
                        taxa_store,
                        identified: &mut identified,
                    });
                }
                if identified {
                    tx_app_message
                        .send(crate::AppMessage::SkipCurrentObservation)
                        .unwrap();
                }
            }
        });
    }
}
