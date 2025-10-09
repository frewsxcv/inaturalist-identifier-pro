use actix::{Actor, SystemService};
use inaturalist::models::{Observation, ShowTaxon};
use inaturalist_oauth::{Authenticator, PkceVerifier};
use inaturalist_pro_actors::{
    ApiFetchCurrentUserMessage, ApiLoaderActor, BuildTaxonTreeMessage, ExchangeCode,
    StartLoadingObservationsMessage, TaxonTreeBuilderActor,
};
use inaturalist_pro_core::{taxon_tree, AppMessage, AppState, QueryResult, TaxaStore};
use inaturalist_pro_ui::Ui;
use oauth2::AuthorizationCode;

type ObservationId = i32;

pub(crate) struct App {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<AppMessage>,
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<AppMessage>,
    pub api_loader_addr: Option<actix::Addr<ApiLoaderActor>>,
    pub observation_grid: Option<inaturalist_pro_geo::GeohashGrid>,
    pub observation_request: Option<inaturalist::apis::observations_api::ObservationsGetParams>,
    pub observation_soft_limit: Option<std::sync::Arc<std::sync::atomic::AtomicI32>>,
    pub oauth_addr: actix::Addr<inaturalist_pro_actors::OauthActor>,
    pub api_token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub pkce_verifier: Option<PkceVerifier>,
    pub state: AppState,
    pub ui: Ui,
}

impl Default for App {
    fn default() -> Self {
        let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel();

        let oauth_addr = inaturalist_pro_actors::OauthActor::new(tx_app_message.clone()).start();

        let ui = Ui::new(tx_app_message.clone());

        Self {
            tx_app_message,
            rx_app_message,
            api_loader_addr: None,
            observation_grid: None,
            observation_request: None,
            observation_soft_limit: None,
            oauth_addr,
            api_token: None,
            client_id: None,
            client_secret: None,
            pkce_verifier: None,
            state: AppState::default(),
            ui,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages();
        self.ui.update(ctx, &mut self.state);
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
            taxon_tree: taxon_tree::TaxonTree::default(),
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
        taxon_tree: taxon_tree::TaxonTree,
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
        if let Err(e) = TaxonTreeBuilderActor::from_registry().try_send(BuildTaxonTreeMessage {
            observation_id: observation.id.unwrap(),
            scores,
        }) {
            tracing::warn!("Failed to send BuildTaxonTreeMessage: {}", e);
        }
    }
}

impl App {
    fn process_messages(&mut self) {
        while let Ok(message) = self.rx_app_message.try_recv() {
            match message {
                AppMessage::Progress
                | AppMessage::TaxonLoaded(_)
                | AppMessage::ObservationLoaded(_)
                | AppMessage::ComputerVisionScoreLoaded(_, _)
                | AppMessage::TaxonTree { .. }
                | AppMessage::SkipCurrentObservation
                | AppMessage::StartLoadingObservations => {
                    // Handle non-auth messages with MessageHandler
                    let mut handler = MessageHandler::new(&mut self.state);
                    match message {
                        AppMessage::Progress => handler.handle_progress(),
                        AppMessage::TaxonLoaded(taxon) => handler.handle_taxon_loaded(taxon),
                        AppMessage::ObservationLoaded(observation) => {
                            handler.handle_observation_loaded(observation)
                        }
                        AppMessage::ComputerVisionScoreLoaded(observation_id, scores) => {
                            handler.handle_cv_scores(observation_id, scores);
                        }
                        AppMessage::TaxonTree {
                            observation_id,
                            taxon_tree,
                        } => {
                            handler.handle_taxon_tree(observation_id, taxon_tree);
                        }
                        AppMessage::SkipCurrentObservation => handler.handle_skip_observation(),
                        AppMessage::StartLoadingObservations => {
                            if let (Some(addr), Some(grid), Some(request), Some(soft_limit)) = (
                                &self.api_loader_addr,
                                &self.observation_grid,
                                &self.observation_request,
                                &self.observation_soft_limit,
                            ) {
                                addr.do_send(StartLoadingObservationsMessage {
                                    grid: grid.clone(),
                                    request: request.clone(),
                                    soft_limit: soft_limit.clone(),
                                });
                            }
                        }
                        _ => {}
                    }
                }
                AppMessage::Authenticated(token) => {
                    tracing::info!("Authentication completed successfully");
                    self.api_token = Some(token.clone());
                    self.state.is_authenticated = true;
                    self.state.show_login_modal = false;
                    self.state.auth_status_message =
                        Some("Successfully authenticated!".to_string());

                    // Fetch user info
                    if let Some(addr) = &self.api_loader_addr {
                        addr.do_send(ApiFetchCurrentUserMessage);
                    }
                }
                AppMessage::AuthError(error) => {
                    tracing::error!("Authentication failed: {}", error);
                    self.state.auth_status_message =
                        Some(format!("Authentication failed: {}", error));
                }
                AppMessage::AuthenticationCodeReceived(code) => {
                    self.exchange_code(code);
                }
                AppMessage::InitiateLogin => self.initiate_login(),
                AppMessage::UserLoaded(user) => {
                    tracing::info!(
                        "User info loaded: {}",
                        user.login.as_ref().map(|s| s.as_str()).unwrap_or("unknown")
                    );
                    self.state.current_user = Some(user);
                }
            }
        }
    }

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
                    let _ = tx.send(AppMessage::AuthenticationCodeReceived(code));
                }
                Err(e) => {
                    tracing::error!("Failed to listen for redirect: {}", e);
                    let _ = tx.send(AppMessage::AuthError(format!(
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

        self.oauth_addr.do_send(ExchangeCode {
            code,
            client_id,
            client_secret,
            pkce_verifier,
        });
    }
}
