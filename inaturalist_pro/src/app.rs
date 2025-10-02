use crate::{
    actors::taxon_tree_builder_actor::{BuildTaxonTreeMessage, TaxonTreeBuilderActor},
    panels::TopPanel,
    taxa_store::TaxaStore,
    views::{IdentifyView, ObservationsView, TaxaView, UsersView},
};
use actix::{Actor, SystemService};
use inaturalist::models::{Observation, ShowTaxon};
use inaturalist_oauth::{Authenticator, PkceVerifier};
use oauth2::AuthorizationCode;

type ObservationId = i32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    Identify,
    Observations,
    Users,
    Taxa,
}

pub struct QueryResult {
    pub observation: Observation,
    pub scores: Option<Vec<inaturalist_fetch::ComputerVisionObservationScore>>,
    pub taxon_tree: crate::taxon_tree::TaxonTree,
}

pub struct AppState {
    pub loaded_geohashes: usize,
    pub results: Vec<QueryResult>,
    pub taxa_store: TaxaStore,
    pub current_observation_id: Option<ObservationId>,
    pub current_view: AppView,
    pub is_authenticated: bool,
    pub show_login_modal: bool,
    pub auth_status_message: Option<String>,
}

struct AppPanels {
    top: TopPanel,
}

struct AppViews {
    identify: IdentifyView,
    observations: ObservationsView,
    users: UsersView,
    taxa: TaxaView,
}

pub(crate) struct App {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub observation_loader_addr:
        Option<actix::Addr<crate::actors::observation_loader_actor::ObservationLoaderActor>>,
    pub oauth_addr: actix::Addr<crate::actors::oauth_actor::OauthActor>,
    pub api_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub pkce_verifier: Option<PkceVerifier>,
    pub state: AppState,
    views: AppViews,
    panels: AppPanels,
}

impl Default for App {
    fn default() -> Self {
        let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel();

        let oauth_addr =
            crate::actors::oauth_actor::OauthActor::new(tx_app_message.clone()).start();

        Self {
            tx_app_message,
            rx_app_message,
            observation_loader_addr: None,
            oauth_addr,
            api_token: None,
            client_id: None,
            client_secret: None,
            pkce_verifier: None,
            state: AppState::default(),
            views: AppViews::default(),
            panels: AppPanels::default(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loaded_geohashes: 0,
            results: Vec::new(),
            taxa_store: TaxaStore::default(),
            current_observation_id: None,
            current_view: AppView::Identify,
            is_authenticated: false,
            show_login_modal: false,
            auth_status_message: None,
        }
    }
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

impl Default for AppPanels {
    fn default() -> Self {
        Self {
            top: TopPanel::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);

        self.process_messages();
        self.render_ui(ctx);

        // Show login modal if needed
        if self.state.show_login_modal {
            self.show_login_modal(ctx);
        }
    }
}

struct MessageHandler<'a> {
    state: &'a mut AppState,
}

impl<'a> MessageHandler<'a> {
    fn new(state: &'a mut AppState) -> Self {
        Self { state }
    }

    fn handle_progress(&mut self) {
        self.state.loaded_geohashes += 1;
    }

    fn handle_taxon_loaded(&mut self, taxon: Box<ShowTaxon>) {
        self.state
            .taxa_store
            .0
            .insert(taxon.id.unwrap(), (&*taxon).into());
    }

    fn handle_observation_loaded(&mut self, observation: Box<Observation>) {
        if self.state.current_observation_id.is_none() {
            self.state.current_observation_id = observation.id;
        }

        self.state.results.push(QueryResult {
            observation: *observation.clone(),
            scores: None,
            taxon_tree: Default::default(),
        });
    }

    fn handle_cv_scores(
        &mut self,
        observation_id: ObservationId,
        scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ) {
        let Some(observation_index) = self.find_observation_index(observation_id) else {
            return;
        };

        Self::build_taxon_tree_async(
            &self.state.results[observation_index].observation,
            scores.clone(),
        );
        self.state.results[observation_index].scores = Some(scores);
        self.sort_observations_by_score();
    }

    fn handle_taxon_tree(
        &mut self,
        observation_id: ObservationId,
        taxon_tree: crate::taxon_tree::TaxonTree,
    ) {
        for result in &mut self.state.results {
            if result.observation.id == Some(observation_id) {
                result.taxon_tree = taxon_tree;
                break;
            }
        }
    }

    fn handle_skip_observation(&mut self) {
        let Some(current_index) = self.get_current_observation_index() else {
            return;
        };

        self.state.results.remove(current_index);
        self.select_next_observation();
    }

    fn find_observation_index(&self, observation_id: ObservationId) -> Option<usize> {
        self.state
            .results
            .iter()
            .position(|result| result.observation.id.unwrap() == observation_id)
    }

    fn get_current_observation_index(&self) -> Option<usize> {
        self.state
            .current_observation_id
            .and_then(|id| self.find_observation_index(id))
    }

    fn select_next_observation(&mut self) {
        self.state.current_observation_id = self
            .state
            .results
            .first()
            .map(|result| result.observation.id.unwrap());
    }

    fn sort_observations_by_score(&mut self) {
        self.state.results.sort_unstable_by(|a, b| {
            let score_a = a
                .scores
                .as_ref()
                .and_then(|scores| scores.first())
                .map_or(0.0, |score| score.combined_score);

            let score_b = b
                .scores
                .as_ref()
                .and_then(|scores| scores.first())
                .map_or(0.0, |score| score.combined_score);

            score_b.partial_cmp(&score_a).unwrap()
        });
    }

    fn build_taxon_tree_async(
        observation: &Observation,
        scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ) {
        TaxonTreeBuilderActor::from_registry()
            .try_send(BuildTaxonTreeMessage {
                observation_id: observation.id.unwrap(),
                scores,
            })
            .unwrap();
    }
}

impl App {
    fn process_messages(&mut self) {
        while let Ok(message) = self.rx_app_message.try_recv() {
            match message {
                crate::AppMessage::Progress
                | crate::AppMessage::TaxonLoaded(_)
                | crate::AppMessage::ObservationLoaded(_)
                | crate::AppMessage::ComputerVisionScoreLoaded(_, _)
                | crate::AppMessage::TaxonTree { .. }
                | crate::AppMessage::SkipCurrentObservation => {
                    // Handle non-auth messages with MessageHandler
                    let mut handler = MessageHandler::new(&mut self.state);
                    match message {
                        crate::AppMessage::Progress => handler.handle_progress(),
                        crate::AppMessage::TaxonLoaded(taxon) => handler.handle_taxon_loaded(taxon),
                        crate::AppMessage::ObservationLoaded(observation) => {
                            handler.handle_observation_loaded(observation)
                        }
                        crate::AppMessage::ComputerVisionScoreLoaded(observation_id, scores) => {
                            handler.handle_cv_scores(observation_id, scores);
                        }
                        crate::AppMessage::TaxonTree {
                            observation_id,
                            taxon_tree,
                        } => {
                            handler.handle_taxon_tree(observation_id, taxon_tree);
                        }
                        crate::AppMessage::SkipCurrentObservation => {
                            handler.handle_skip_observation()
                        }
                        _ => {}
                    }
                }
                crate::AppMessage::Authenticated(token) => {
                    tracing::info!("Authentication completed successfully");
                    self.api_token = Some(token);
                    self.state.is_authenticated = true;
                    self.state.show_login_modal = false;
                    self.state.auth_status_message =
                        Some("Successfully authenticated!".to_string());
                }
                crate::AppMessage::AuthError(error) => {
                    tracing::error!("Authentication failed: {}", error);
                    self.state.auth_status_message =
                        Some(format!("Authentication failed: {}", error));
                }
                crate::AppMessage::AuthenticationCodeReceived(code) => {
                    self.exchange_code(code);
                }
            }
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        self.panels.top.show(
            ctx,
            self.state.is_authenticated,
            &mut self.state.show_login_modal,
            &mut self.state.auth_status_message,
        );

        egui::SidePanel::left("navigation_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("iNaturalist Pro");
                ui.separator();

                ui.selectable_value(
                    &mut self.state.current_view,
                    AppView::Identify,
                    "🔍 Identify",
                );
                ui.selectable_value(
                    &mut self.state.current_view,
                    AppView::Observations,
                    "📷 Observations",
                );
                ui.selectable_value(&mut self.state.current_view, AppView::Users, "👤 Users");
                ui.selectable_value(&mut self.state.current_view, AppView::Taxa, "🌿 Taxa");
            });

        match self.state.current_view {
            AppView::Identify => self.render_identify_view(ctx),
            AppView::Observations => self.render_observations_view(ctx),
            AppView::Users => self.render_users_view(ctx),
            AppView::Taxa => self.render_taxa_view(ctx),
        }
    }

    fn render_identify_view(&mut self, ctx: &egui::Context) {
        self.views.identify.show(
            ctx,
            &self.state.results,
            self.state.current_observation_id,
            &self.state.taxa_store,
            &self.tx_app_message,
            self.state.loaded_geohashes,
            self.observation_loader_addr.as_ref(),
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

    fn show_login_modal(&mut self, ctx: &egui::Context) {
        egui::Window::new("Login to iNaturalist")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("To use features that require authentication,");
                ui.label("you need to log in to your iNaturalist account.");
                ui.add_space(10.0);

                if let Some(msg) = &self.state.auth_status_message {
                    ui.colored_label(egui::Color32::RED, msg);
                    ui.add_space(10.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("Login").clicked() {
                        self.initiate_login();
                    }
                    if ui.button("Cancel").clicked() {
                        self.state.show_login_modal = false;
                        self.state.auth_status_message = None;
                    }
                });

                ui.add_space(10.0);
                ui.label("Note: Login will open your browser for OAuth authentication.");
            });
    }

    // NOTE: The AppMessage enum needs to be updated with AuthenticationCodeReceived
    // in lib.rs for this to compile.
    fn initiate_login(&mut self) {
        let client_id = match &self.client_id {
            Some(id) => id.clone(),
            None => {
                self.state.auth_status_message = Some("Authentication not available".to_string());
                return;
            }
        };

        let client_secret = match &self.client_secret {
            Some(secret) => secret.clone(),
            None => {
                self.state.auth_status_message = Some("Authentication not available".to_string());
                return;
            }
        };

        let authenticator = Authenticator::new(client_id.clone(), client_secret.clone());
        let auth_info = match authenticator.authorization_url() {
            Ok(info) => info,
            Err(e) => {
                let msg = format!("Failed to start auth: {}", e);
                tracing::error!("{}", msg);
                self.state.auth_status_message = Some(msg);
                return;
            }
        };

        // Store verifier for later
        self.pkce_verifier = Some(auth_info.pkce_verifier);

        // Open browser on main thread
        if let Err(e) = opener::open(auth_info.url.to_string()) {
            let msg = format!("Failed to open browser: {}", e);
            tracing::error!("{}", msg);
            self.state.auth_status_message = Some(msg);
            return;
        }

        self.state.auth_status_message =
            Some("Waiting for you to log in in your browser...".to_string());

        let tx = self.tx_app_message.clone();

        // Spawn a blocking task to listen for the redirect, since it's a blocking operation.
        tokio::task::spawn_blocking(move || {
            let authenticator = Authenticator::new(client_id, client_secret);
            match authenticator.listen_for_redirect() {
                Ok(code) => {
                    let _ = tx.send(crate::AppMessage::AuthenticationCodeReceived(code));
                }
                Err(e) => {
                    tracing::error!("Failed to listen for redirect: {}", e);
                    let _ = tx.send(crate::AppMessage::AuthError(format!(
                        "Login failed: {}. Please try again.",
                        e
                    )));
                }
            }
        });
    }

    fn exchange_code(&mut self, code: AuthorizationCode) {
        let client_id = self.client_id.clone().unwrap();
        let client_secret = self.client_secret.clone().unwrap();
        let pkce_verifier = self.pkce_verifier.take().unwrap();

        self.state.auth_status_message =
            Some("Authentication successful, fetching API token...".to_string());

        self.oauth_addr
            .do_send(crate::actors::oauth_actor::ExchangeCode {
                code,
                client_id,
                client_secret,
                pkce_verifier,
            });
    }
}
