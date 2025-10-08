use egui::{RichText, ScrollArea};

#[derive(Default)]
pub struct UsersView {
    search_query: String,
    current_user: Option<UserProfile>,
    loading: bool,
}

#[derive(Debug, Clone)]
struct UserProfile {
    id: i32,
    login: String,
    name: Option<String>,
    icon_url: Option<String>,
    created_at: String,
    observations_count: i32,
    identifications_count: i32,
    species_count: i32,
    journal_posts_count: i32,
    activity_count: i32,
    site_id: i32,
    roles: Vec<String>,
}

impl UsersView {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("👤 User Lookup");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                self.render_search_form(ui);
                ui.add_space(20.0);
                self.render_user_profile(ui);
            });
        });
    }

    fn render_search_form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.search_query);

            if ui.button("🔍 Search").clicked() {
                self.perform_search();
            }

            if ui.button("Clear").clicked() {
                self.clear();
            }
        });

        ui.add_space(5.0);
        ui.label(
            RichText::new("Enter an iNaturalist username to view their profile")
                .small()
                .weak(),
        );
    }

    fn render_user_profile(&mut self, ui: &mut egui::Ui) {
        if self.loading {
            ui.spinner();
            ui.label("Loading user profile...");
            return;
        }

        let Some(user) = &self.current_user else {
            ui.label(
                RichText::new("No user profile to display. Search for a user above.")
                    .italics()
                    .weak(),
            );
            return;
        };

        // User header
        ui.horizontal(|ui| {
            // TODO: Display user icon when available
            if let Some(_icon_url) = &user.icon_url {
                ui.label("🖼️");
            }

            ui.vertical(|ui| {
                ui.heading(&user.login);
                if let Some(name) = &user.name {
                    ui.label(RichText::new(name).size(14.0));
                }
                ui.label(
                    RichText::new(format!("User ID: {}", user.id))
                        .small()
                        .weak(),
                );
            });
        });

        ui.add_space(15.0);
        ui.separator();

        // Stats section
        ui.heading("Statistics");
        ui.add_space(10.0);

        egui::Grid::new("user_stats_grid")
            .num_columns(2)
            .spacing([40.0, 8.0])
            .show(ui, |ui| {
                self.render_stat(ui, "📷 Observations", user.observations_count);
                ui.end_row();

                self.render_stat(ui, "🔍 Identifications", user.identifications_count);
                ui.end_row();

                self.render_stat(ui, "🌿 Species", user.species_count);
                ui.end_row();

                self.render_stat(ui, "📝 Journal Posts", user.journal_posts_count);
                ui.end_row();

                self.render_stat(ui, "💫 Total Activity", user.activity_count);
                ui.end_row();
            });

        ui.add_space(15.0);
        ui.separator();

        // Additional info
        ui.heading("Details");
        ui.add_space(10.0);

        egui::Grid::new("user_details_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label("Member since:");
                ui.label(&user.created_at);
                ui.end_row();

                ui.label("Site ID:");
                ui.label(format!("{}", user.site_id));
                ui.end_row();

                if !user.roles.is_empty() {
                    ui.label("Roles:");
                    ui.label(user.roles.join(", "));
                    ui.end_row();
                }
            });
    }

    fn render_stat(&self, ui: &mut egui::Ui, label: &str, count: i32) {
        ui.label(label);
        ui.label(RichText::new(format!("{}", count)).strong());
    }

    fn perform_search(&mut self) {
        // TODO: Implement actual API search
        self.loading = false;
        // This is a placeholder - in a real implementation, this would:
        // 1. Send a message to an actor to fetch user data
        // 2. Update self.current_user when the data arrives

        // For now, show example data when searching
        if !self.search_query.is_empty() {
            self.current_user = Some(UserProfile {
                id: 12345,
                login: self.search_query.clone(),
                name: Some("Example User".to_string()),
                icon_url: None,
                created_at: "2020-01-15".to_string(),
                observations_count: 1234,
                identifications_count: 5678,
                species_count: 890,
                journal_posts_count: 12,
                activity_count: 6924,
                site_id: 1,
                roles: vec![],
            });
        }
    }

    fn clear(&mut self) {
        self.search_query.clear();
        self.current_user = None;
    }
}
