use crate::{
    panels::{DetailsPanel, IdentificationPanel, ObservationGalleryPanel},
    taxa_store::TaxaStore,
    AppMessage,
};
use egui::RichText;

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
        results: &[crate::app::QueryResult],
        current_observation_id: Option<i32>,
        taxa_store: &TaxaStore,
        tx_app_message: &tokio::sync::mpsc::UnboundedSender<AppMessage>,
        loaded_geohashes: usize,
        observation_loader_addr: Option<&actix::Addr<crate::actors::ObservationLoaderActor>>,
    ) {
        // Show loading screen if no results yet
        if results.is_empty() {
            self.show_loading_screen(
                ctx,
                loaded_geohashes,
                tx_app_message,
                observation_loader_addr,
            );
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
        _tx_app_message: &tokio::sync::mpsc::UnboundedSender<AppMessage>,
        observation_loader_addr: Option<&actix::Addr<crate::actors::ObservationLoaderActor>>,
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

                        if let Some(addr) = observation_loader_addr {
                            use crate::actors::observation_loader_actor::StartLoadingMessage;

                            if let Err(e) = addr.try_send(StartLoadingMessage) {
                                tracing::error!("Failed to start observation loading: {}", e);
                            }
                        } else {
                            tracing::error!("Observation loader actor not available");
                        }
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
