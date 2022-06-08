pub(crate) struct TemplateApp {
    pub display_string: Vec<String>,
    pub rx_progress: async_channel::Receiver<crate::Progress>,
    pub loaded_geohashes: usize,
    pub total_geohashes: usize,
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(_) = self.rx_progress.try_recv() {
            self.loaded_geohashes += 1;
        }

        // Redraw every 1 second
        let cloned_ctx = ctx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
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
                    ui.add(
                        egui::ProgressBar::new(
                            self.loaded_geohashes as f32 / self.total_geohashes as f32,
                        )
                    );
                } else {
                    ui.heading("Results");
                    for url in &self.display_string {
                        ui.hyperlink(url);
                    }
                }
            });
        });
    }
}
