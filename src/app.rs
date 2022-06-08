use inaturalist::models::Observation;
use std::{sync, thread};

pub(crate) struct TemplateApp {
    pub rx_app_message: async_channel::Receiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub total_geohashes: usize,
    pub results: Vec<Observation>,
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(app_message) = self.rx_app_message.try_recv() {
            match app_message {
                crate::AppMessage::Progress => {
                    self.loaded_geohashes += 1;
                }
                crate::AppMessage::Results(observations) => {
                    let image_store = self.image_store.clone();
                    self.results = observations.clone();
                    thread::spawn(move || {
                        for observation in observations {
                            if let Some(photos) = &observation.photos {
                                let image_url =
                                    photos[0].url.as_ref().unwrap().replace("square", "medium");
                                let request = ehttp::Request::get(image_url);
                                let image_store = image_store.clone();
                                ehttp::fetch(request, move |response| {
                                    let image = response.and_then(parse_response);
                                    image_store
                                        .write()
                                        .unwrap()
                                        .insert(observation.id.unwrap(), image.unwrap());
                                    // ctx.request_repaint();
                                });
                            }
                        }
                        // image_store.begin_loading(results);
                    });
                }
            }
        }

        // Redraw every 1 second
        let cloned_ctx = ctx.clone();
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_secs(1));
            cloned_ctx.request_repaint();
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(label);
            // });

            // ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     *value += 1.0;
            // }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.loaded_geohashes < self.total_geohashes {
                    ui.heading("Loading data");
                    ui.add(egui::ProgressBar::new(
                        self.loaded_geohashes as f32 / self.total_geohashes as f32,
                    ));
                } else {
                    ui.heading("Results");
                    for observation in &self.results {
                        ui.hyperlink(observation.uri.as_ref().unwrap());
                        if let Some(image) = self
                            .image_store
                            .read()
                            .unwrap()
                            .load(observation.id.unwrap())
                        {
                            image.show(ui);
                        } else {
                            ui.spinner();
                        }
                        ui.separator();
                    }
                }
            });
        });
    }
}

fn parse_response(response: ehttp::Response) -> Result<egui_extras::RetainedImage, String> {
    let content_type = response.content_type().unwrap_or_default();
    if content_type.starts_with("image/") {
        egui_extras::RetainedImage::from_image_bytes(&response.url, &response.bytes)
    } else {
        Err(format!(
            "Expected image, found content-type {:?}",
            content_type
        ))
    }
}
