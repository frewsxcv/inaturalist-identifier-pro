use crate::app::QueryResult;

pub struct ObservationGalleryWidget<'a> {
    pub results: &'a [QueryResult],
}

impl<'a> egui::Widget for ObservationGalleryWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.heading("Observation Gallery");
            ui.label(format!("Loaded observations: {}", self.results.len()));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for result in self.results {
                    if let Some(photo) = result.observation.photos.as_ref().and_then(|p| p.first())
                    {
                        if let Some(url) = photo.url.as_ref() {
                            ui.image(url);
                        }
                    }
                }
            });
        })
        .response
    }
}
