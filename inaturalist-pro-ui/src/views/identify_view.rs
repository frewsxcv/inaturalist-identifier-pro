use crate::panels::{DetailsPanel, IdentificationPanel, ObservationGalleryPanel};
use egui::RichText;
use inaturalist_pro_core::{AppMessage, QueryResult, TaxaStore};

pub struct IdentifyView {
    details_panel: DetailsPanel,
    identification_panel: IdentificationPanel,
    observation_gallery_panel: ObservationGalleryPanel,
    loading_started: bool,
}

impl Default for IdentifyView {
    fn default() -> Self {
        Self {
            details_panel: DetailsPanel::default(),
            identification_panel: IdentificationPanel::default(),
            observation_gallery_panel: ObservationGalleryPanel::default(),
            loading_started: false,
        }
    }
}

impl IdentifyView {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        results: &[QueryResult],
        current_observation_id: Option<i32>,
        taxa_store: &TaxaStore,
        tx_app_message: &tokio::sync::mpsc::UnboundedSender<AppMessage>,
        loaded_geohashes: usize,
    ) {
        // Show loading screen if no results yet
        if results.is_empty() {
            self.show_loading_screen(ctx, loaded_geohashes, tx_app_message);
            return;
        }

        let current_observation_index = current_observation_id.and_then(|id| {
            results
                .iter()
                .position(|result| result.observation.id.unwrap() == id)
        });

        self.observation_gallery_panel.show(ctx, results);

        let current_observation = current_observation_index.map(|index| &results[index]);

        self.identification_panel
            .show(ctx, current_observation, taxa_store, tx_app_message);
        self.details_panel.show(ctx, current_observation);
    }

    fn show_loading_screen(
        &mut self,
        ctx: &egui::Context,
        loaded_geohashes: usize,
        tx_app_message: &tokio::sync::mpsc::UnboundedSender<AppMessage>,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);

                ui.heading("🔍 Identify Mode");
                ui.add_space(20.0);

                if !self.loading_started {
                    ui.label(
                        "Click the button below to start loading observations from iNaturalist.",
                    );
                    ui.add_space(20.0);

                    if ui.button("▶ Start Loading Observations").clicked() {
                        self.loading_started = true;
                        let _ = tx_app_message.send(AppMessage::StartLoadingObservations);
                    }
                } else if loaded_geohashes == 0 {
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.label("Initializing...");
                } else {
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(format!(
                            "Loading observations... ({} regions loaded)",
                            loaded_geohashes
                        ))
                        .size(16.0),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("This may take a moment on first launch")
                            .weak()
                            .italics(),
                    );
                }

                ui.add_space(30.0);
                ui.label("Once loaded, you'll be able to:");
                ui.label("• Browse unidentified observations");
                ui.label("• View computer vision suggestions");
                ui.label("• Help identify species");
            });
        });
    }
}
