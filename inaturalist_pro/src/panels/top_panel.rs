#[derive(Default)]
pub struct TopPanel;

impl TopPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        is_authenticated: bool,
        show_login_modal: &mut bool,
        auth_status_message: &mut Option<String>,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // Left side: File menu
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Spacer to push profile button to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Profile/Auth button
                    if is_authenticated {
                        self.show_authenticated_menu(ui, auth_status_message);
                    } else {
                        self.show_login_button(ui, show_login_modal, auth_status_message);
                    }
                });
            });
        });
    }

    fn show_authenticated_menu(&self, ui: &mut egui::Ui, auth_status_message: &mut Option<String>) {
        ui.menu_button("👤 Profile", |ui| {
            ui.label("✅ Logged in");
            ui.separator();

            if ui.button("Account Info").clicked() {
                *auth_status_message = Some("Account info coming soon!".to_string());
            }

            if ui.button("Logout").clicked() {
                *auth_status_message = Some("Logout functionality coming soon!".to_string());
            }
        });
    }

    fn show_login_button(
        &self,
        ui: &mut egui::Ui,
        show_login_modal: &mut bool,
        auth_status_message: &mut Option<String>,
    ) {
        // Show status message if present
        if let Some(msg) = auth_status_message {
            if msg.contains("Success") {
                ui.colored_label(egui::Color32::GREEN, "✓");
            }
        }

        // Login button
        if ui.button("🔒 Login").clicked() {
            *show_login_modal = true;
            *auth_status_message = None;
        }
    }
}
