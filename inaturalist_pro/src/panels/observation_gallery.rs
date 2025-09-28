use crate::app::QueryResult;

#[derive(Default)]
pub struct ObservationGalleryPanel;

impl ObservationGalleryPanel {
    pub fn show(&mut self, ctx: &egui::Context, results: &[QueryResult]) {
        egui::SidePanel::left("observation_gallery").show(ctx, |ui| {
            ui.heading("Observation Gallery");
            ui.label(format!("Loaded observations: {}", results.len()));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for result in results {
                    if let Some(photo) = result.observation.photos.as_ref().and_then(|p| p.first())
                    {
                        if let Some(url) = photo.url.as_ref() {
                            ui.image(url);
                        }
                    }
                }
            });
        });
    }
}
