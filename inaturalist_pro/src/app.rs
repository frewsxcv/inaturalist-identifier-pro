use crate::{
    panels::{DetailsPanel, IdentificationPanel, ObservationGalleryPanel, TopPanel},
    taxa_store::TaxaStore,
    taxon_tree_builder_actor::{BuildTaxonTreeMessage, TaxonTreeBuilderActor},
};
use actix::SystemService;
use inaturalist::models::Observation;

type ObservationId = i32;

pub(crate) struct App {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
    pub rx_app_message: tokio::sync::mpsc::UnboundedReceiver<crate::AppMessage>,
    pub loaded_geohashes: usize,
    pub results: Vec<QueryResult>,
    pub taxa_store: TaxaStore,
    pub current_observation_id: Option<ObservationId>,

    // Panels
    details_panel: DetailsPanel,
    identification_panel: IdentificationPanel,
    observation_gallery_panel: ObservationGalleryPanel,
    top_panel: TopPanel,
}

impl Default for App {
    fn default() -> Self {
        let (tx_app_message, rx_app_message) = tokio::sync::mpsc::unbounded_channel();
        Self {
            tx_app_message,
            rx_app_message,
            loaded_geohashes: 0,
            results: Default::default(),
            taxa_store: Default::default(),
            current_observation_id: Default::default(),
            details_panel: Default::default(),
            identification_panel: Default::default(),
            observation_gallery_panel: Default::default(),
            top_panel: Default::default(),
        }
    }
}

pub struct QueryResult {
    pub observation: Observation,
    pub scores: Option<Vec<inaturalist_fetch::ComputerVisionObservationScore>>,
    pub taxon_tree: crate::taxon_tree::TaxonTree,
}

impl App {
    fn build_taxon_tree_in_background_thread(
        &self,
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        if let Ok(app_message) = self.rx_app_message.try_recv() {
            match app_message {
                crate::AppMessage::Progress => {
                    self.loaded_geohashes += 1;
                }
                crate::AppMessage::TaxonLoaded(taxon) => {
                    self.taxa_store
                        .0
                        .insert(taxon.id.unwrap(), (&*taxon).into());
                }
                crate::AppMessage::ObservationLoaded(observation) => {
                    if self.current_observation_id.is_none() {
                        self.current_observation_id = observation.id;
                    }

                    self.results.push(QueryResult {
                        observation: *observation.clone(),
                        scores: None,
                        taxon_tree: Default::default(),
                    });
                }
                crate::AppMessage::ComputerVisionScoreLoaded(observation_id, scores) => {
                    let Some(observation_index) =
                        self.find_index_for_observation_id(observation_id)
                    else {
                        // TODO: Log error here
                        return;
                    };
                    self.build_taxon_tree_in_background_thread(
                        &self.results[observation_index].observation,
                        scores.clone(),
                    );
                    self.results[observation_index].scores = Some(scores);
                    self.sort_results();
                }
                crate::AppMessage::TaxonTree {
                    observation_id,
                    taxon_tree,
                } => {
                    for n in &mut self.results {
                        if n.observation.id == Some(observation_id) {
                            n.taxon_tree = taxon_tree;
                            break;
                        }
                    }
                }
                crate::AppMessage::SkipCurrentObservation => {
                    let Some(current_observation_index) = self.find_index_for_current_observation()
                    else {
                        return;
                    };
                    self.results.remove(current_observation_index);
                    self.select_new_observation();
                }
            }
        }

        self.top_panel.show(ctx);
        self.observation_gallery_panel.show(ctx, &self.results);

        let query_result = self
            .find_index_for_current_observation()
            .map(|index| &self.results[index]);

        self.identification_panel
            .show(ctx, query_result, &self.taxa_store, &self.tx_app_message);
        self.details_panel.show(ctx, query_result);
    }
}

impl App {
    fn select_new_observation(&mut self) {
        self.current_observation_id = self.results.first().map(|o| o.observation.id.unwrap());
    }

    fn find_index_for_current_observation(&self) -> Option<usize> {
        self.current_observation_id
            .and_then(|id| self.find_index_for_observation_id(id))
    }

    fn find_index_for_observation_id(&self, observation_id: ObservationId) -> Option<usize> {
        self.results
            .iter()
            .enumerate()
            .find(|(_, result)| result.observation.id.unwrap() == observation_id)
            .map(|(i, _)| i)
    }

    fn sort_results(&mut self) {
        self.results.sort_unstable_by(|a, b| {
            let score_a = a
                .scores
                .as_ref()
                .map_or(0., |scores| scores[0].combined_score);
            let score_b = b
                .scores
                .as_ref()
                .map_or(0., |scores| scores[0].combined_score);
            score_a.partial_cmp(&score_b).unwrap().reverse()
        });
    }
}
