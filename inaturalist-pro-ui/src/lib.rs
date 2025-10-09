pub mod panels;

pub mod views;

pub mod widgets;

pub mod utils;

use crate::{
    panels::TopPanel,
    views::{IdentifyView, ObservationsView, TaxaView, UsersView},
};

use eframe::egui;
use inaturalist_pro_core::{AppMessage, AppState, AppView};

use tokio::sync::mpsc::UnboundedSender;

struct AppPanels {
    top: TopPanel,
}

impl Default for AppPanels {
    fn default() -> Self {
        Self {
            top: TopPanel::default(),
        }
    }
}

struct AppViews {
    identify: IdentifyView,
    observations: ObservationsView,
    users: UsersView,
    taxa: TaxaView,
}

impl Default for AppViews {
    fn default() -> Self {
        Self {
            identify: IdentifyView::default(),
            observations: ObservationsView::default(),
            users: UsersView::default(),
            taxa: TaxaView::default(),
        }
    }
}

pub struct Ui {
    views: AppViews,
    panels: AppPanels,
    tx_app_message: UnboundedSender<AppMessage>,
}

impl Ui {
    pub fn new(tx_app_message: UnboundedSender<AppMessage>) -> Self {
        Self {
            views: AppViews::default(),
            panels: AppPanels::default(),
            tx_app_message,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui_extras::install_image_loaders(ctx);

        self.render_ui(ctx, state);

        if state.show_login_modal {
            self.show_login_modal(ctx, state);
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context, state: &mut AppState) {
        self.panels.top.show(
            ctx,
            state.is_authenticated,
            &mut state.show_login_modal,
            &mut state.auth_status_message,
            &state.current_user,
        );

        egui::SidePanel::left("navigation_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("iNaturalist Pro");
                ui.separator();

                ui.selectable_value(&mut state.current_view, AppView::Identify, "🔍 Identify");
                ui.selectable_value(
                    &mut state.current_view,
                    AppView::Observations,
                    "📷 Observations",
                );
                ui.selectable_value(&mut state.current_view, AppView::Users, "👤 Users");
                ui.selectable_value(&mut state.current_view, AppView::Taxa, "🌿 Taxa");
            });

        match state.current_view {
            AppView::Identify => self.render_identify_view(ctx, state),
            AppView::Observations => self.render_observations_view(ctx),
            AppView::Users => self.render_users_view(ctx),
            AppView::Taxa => self.render_taxa_view(ctx),
        }
    }

    fn render_identify_view(&mut self, ctx: &egui::Context, state: &AppState) {
        self.views.identify.show(
            ctx,
            &state.results,
            state.current_observation_id,
            &state.taxa_store,
            &self.tx_app_message,
            state.loaded_geohashes,
        );
    }

    fn render_observations_view(&mut self, ctx: &egui::Context) {
        self.views.observations.show(ctx);
    }

    fn render_users_view(&mut self, ctx: &egui::Context) {
        self.views.users.show(ctx);
    }

    fn render_taxa_view(&mut self, ctx: &egui::Context) {
        self.views.taxa.show(ctx);
    }

    fn show_login_modal(&mut self, ctx: &egui::Context, state: &mut AppState) {
        egui::Window::new("Login to iNaturalist")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("To use features that require authentication,");
                ui.label("you need to log in to your iNaturalist account.");
                ui.add_space(10.0);

                if let Some(msg) = &state.auth_status_message {
                    ui.colored_label(egui::Color32::RED, msg);
                    ui.add_space(10.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("Login").clicked() {
                        let _ = self.tx_app_message.send(AppMessage::InitiateLogin);
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_login_modal = false;
                        state.auth_status_message = None;
                    }
                });

                ui.add_space(10.0);
                ui.label("Note: Login will open your browser for OAuth authentication.");
            });
    }
}
