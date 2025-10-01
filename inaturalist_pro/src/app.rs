use crate::{
    actors::taxon_tree_builder_actor::{BuildTaxonTreeMessage, TaxonTreeBuilderActor},
    panels::TopPanel,
    taxa_store::TaxaStore,
    views::{IdentifyView, ObservationsView, TaxaView, UsersView},
};
use actix::SystemService;
use inaturalist::models::{Observation, ShowTaxon};

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

struct AppState {
    loaded_geohashes: usize,
    results: Vec<QueryResult>,
    taxa_store: TaxaStore,
    current_observation_id: Option<ObservationId>,
    current_view: AppView,
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
    pub observation_loader_addr: Option<actix::Addr<crate::actors::ObservationLoaderActor>>,
    state: AppState,
    views: AppViews,
    panels: AppPanels,
}

impl Default for App {
    fn default() -> Self {
        let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel();
        Self {
            tx_app_message,
            rx_app_message,
            observation_loader_addr: None,
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
        let mut handler = MessageHandler::new(&mut self.state);

        while let Ok(message) = self.rx_app_message.try_recv() {
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
                crate::AppMessage::SkipCurrentObservation => handler.handle_skip_observation(),
            }
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        self.panels.top.show(ctx);

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
}
