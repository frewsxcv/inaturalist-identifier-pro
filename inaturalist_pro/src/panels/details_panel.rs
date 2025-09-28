use crate::app::QueryResult;

#[derive(Default)]
pub struct DetailsPanel;

impl DetailsPanel {
    pub fn show(&mut self, ctx: &egui::Context, query_result: Option<&QueryResult>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Details Panel");
            let Some(query_result) = query_result else {
                // TODO: Show a message here?
                return;
            };

            let _rect = ui.max_rect();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(image_url) = query_result
                    .observation
                    .photos
                    .as_ref()
                    .and_then(|photos| photos.first())
                    .and_then(|photo| photo.url.as_ref())
                {
                    let image_size =
                        egui::Vec2::new(ui.available_width(), ui.available_height() * 0.6);
                    let _response = ui.add(egui::Image::new(image_url).max_size(image_size));
                }

                if let Some(uri) = &query_result.observation.uri {
                    ui.hyperlink(uri);
                }
                if let Some(observed_on) = &query_result.observation.observed_on_string {
                    ui.label(format!("Observed on: {}", observed_on));
                }
                if let Some(place_guess) = &query_result.observation.place_guess {
                    ui.label(format!("Location: {}", place_guess));
                }
                if let Some(description) = &query_result.observation.description {
                    ui.label("Description:");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(description);
                    });
                }
                ui.separator();
            });
        });
    }
}
