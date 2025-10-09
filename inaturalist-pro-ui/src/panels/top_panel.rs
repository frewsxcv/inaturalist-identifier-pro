use inaturalist::models::User;

#[derive(Default)]
pub struct TopPanel;

impl TopPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        is_authenticated: bool,
        show_login_modal: &mut bool,
        auth_status_message: &mut Option<String>,
        current_user: &Option<User>,
        pending_api_requests: usize,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // Left side: File menu
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Show API request loading spinner
                if pending_api_requests > 0 {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(format!(
                                "{} API requests queued",
                                pending_api_requests
                            ))
                            .color(egui::Color32::from_rgb(100, 150, 255))
                            .italics(),
                        );
                    });
                }

                // Spacer to push profile button to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Profile/Auth button
                    if is_authenticated {
                        self.show_authenticated_menu(ui, auth_status_message, current_user);
                    } else {
                        self.show_login_button(ui, show_login_modal, auth_status_message);
                    }
                });
            });
        });
    }

    fn show_authenticated_menu(
        &self,
        ui: &mut egui::Ui,
        auth_status_message: &mut Option<String>,
        current_user: &Option<User>,
    ) {
        // Check if user data is loaded
        if let Some(user) = current_user {
            // User data is loaded, show username and icon
            let username = user.login.as_ref().map(|s| s.as_str()).unwrap_or("User");

            let icon_url = user.icon.as_ref().and_then(|url| {
                if url.is_empty() {
                    None
                } else {
                    Some(url.as_str())
                }
            });

            // Show profile picture and username in button
            ui.horizontal(|ui| {
                if let Some(url) = icon_url {
                    ui.add(egui::Image::new(url).max_height(20.0).corner_radius(10.0));
                } else {
                    ui.label("👤");
                }
                ui.menu_button(username, |ui| {
                    // Show larger profile picture in dropdown if available
                    if let Some(url) = icon_url {
                        ui.horizontal(|ui| {
                            ui.add(egui::Image::new(url).max_width(40.0).corner_radius(20.0));
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(username).strong());
                                ui.label("✅ Logged in");
                            });
                        });
                        ui.separator();
                    } else {
                        ui.label("✅ Logged in");
                        ui.separator();
                    }

                    if ui.button("Account Info").clicked() {
                        *auth_status_message = Some("Account info coming soon!".to_string());
                    }

                    if ui.button("Logout").clicked() {
                        *auth_status_message =
                            Some("Logout functionality coming soon!".to_string());
                    }
                });
            });
        } else {
            // User data is still loading, show spinner with prominent styling
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new("Loading user info...")
                        .color(egui::Color32::from_rgb(100, 150, 255))
                        .italics(),
                );
            });
        }
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
