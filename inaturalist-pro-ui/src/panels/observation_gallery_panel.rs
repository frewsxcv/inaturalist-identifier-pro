use crate::widgets::observation_gallery::ObservationGalleryWidget;
use inaturalist_pro_core::QueryResult;

#[derive(Default)]
pub struct ObservationGalleryPanel;

impl ObservationGalleryPanel {
    pub fn show(&mut self, ctx: &egui::Context, results: &[QueryResult]) {
        egui::SidePanel::left("observation_gallery").show(ctx, |ui| {
            ui.add(ObservationGalleryWidget { results });
        });
    }
}
