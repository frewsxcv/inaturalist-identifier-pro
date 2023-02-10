use inaturalist::models::Observation;
use std::sync;

pub(crate) struct TemplateApp {
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub total_geohashes: usize,
    pub results: Vec<Foo>,
    pub image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
}

pub struct Foo {
    observation: Observation,
    score: f32,
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(app_message) = self.rx_app_message.try_recv() {
            match app_message {
                crate::AppMessage::Progress => {
                    self.loaded_geohashes += 1;
                }
                crate::AppMessage::Result((observation, score)) => {
                    let image_store = self.image_store.clone();
                    self.results.push(Foo {
                        observation: observation.clone(),
                        score,
                    });
                    self.results
                        .sort_unstable_by(|a, b| a.score.partial_cmp(&b.score).unwrap().reverse());
                    tokio::spawn(async move {
                        if let Some(photo_url) = observation
                            .photos
                            .as_ref()
                            .and_then(|p| p.get(0).map(|p| p.url.to_owned()))
                        {
                            let image_url = photo_url.as_ref().unwrap().replace("square", "medium");
                            let request = ehttp::Request::get(image_url);
                            let image_store = image_store.clone();
                            fetch_image(request, image_store, observation);
                        }
                        // image_store.begin_loading(results);
                    });
                }
            }
        }

        // Redraw every 1 second
        let cloned_ctx = ctx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            cloned_ctx.request_repaint();
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
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
                }
                ui.heading("Results");
                for foo in &self.results {
                    ui.hyperlink(foo.observation.uri.as_ref().unwrap());
                    if let Some(image) = self
                        .image_store
                        .read()
                        .unwrap()
                        .load(foo.observation.id.unwrap())
                    {
                        image.show(ui);
                        ui.label(format!("Score: {}", foo.score));
                    } else {
                        ui.spinner();
                    }
                    ui.separator();
                }
            });
        });
    }
}

fn parse_image_response(response: ehttp::Response) -> Result<egui_extras::RetainedImage, String> {
    let content_type = response.content_type().unwrap_or_default();
    if content_type.starts_with("image/") {
        egui_extras::RetainedImage::from_image_bytes(&response.url, &response.bytes)
    } else {
        Err(format!(
            "Expected image, found content-type {content_type:?}"
        ))
    }
}

fn fetch_image(
    request: ehttp::Request,
    image_store: sync::Arc<sync::RwLock<crate::image_store::ImageStore>>,
    observation: Observation,
) {
    ehttp::fetch(request, move |response| {
        let image = response.and_then(parse_image_response);
        image_store
            .write()
            .unwrap()
            .insert(observation.id.unwrap(), image.unwrap());
        // ctx.request_repaint();
    });
}
