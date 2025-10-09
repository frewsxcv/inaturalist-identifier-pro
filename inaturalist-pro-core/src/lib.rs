use inaturalist::models::{Observation, ObservationTaxon, ShowTaxon, User};
use oauth2::AuthorizationCode;
use serde::{Deserialize, Serialize};
use std::collections;

// Type Aliases
pub type ObservationId = i32;
pub type TaxaId = i32;

// --- AppView ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppView {
    Identify,
    Observations,
    Users,
    Taxa,
}

impl Default for AppView {
    fn default() -> Self {
        AppView::Identify
    }
}

// --- TaxaStore & Taxon ---
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TaxaStore(pub collections::HashMap<TaxaId, Taxon>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Taxon {
    pub id: TaxaId,
    pub name: String,
}

impl From<&ShowTaxon> for Taxon {
    fn from(value: &ShowTaxon) -> Self {
        Taxon {
            id: value.id.unwrap_or_default(),
            name: value
                .preferred_common_name
                .clone()
                .or_else(|| value.name.clone())
                .unwrap_or_default(),
        }
    }
}

impl From<&ObservationTaxon> for Taxon {
    fn from(value: &ObservationTaxon) -> Self {
        Taxon {
            id: value.id.unwrap_or_default(),
            name: value
                .preferred_common_name
                .clone()
                .or_else(|| value.name.clone())
                .unwrap_or_default(),
        }
    }
}

// --- TaxonTree ---
pub mod taxon_tree {
    use super::{TaxaId, Taxon};
    use serde::{Deserialize, Serialize};
    use std::collections;

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TaxonTree {
        pub nodes: collections::HashMap<TaxaId, TaxonNode>,
    }

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct TaxonNode {
        pub taxon: Taxon,
        pub children: Vec<TaxaId>,
    }
}

// --- QueryResult ---
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub observation: Observation,
    pub scores: Option<Vec<inaturalist_fetch::ComputerVisionObservationScore>>,
    pub taxon_tree: taxon_tree::TaxonTree,
}

// --- AppState ---
#[derive(Debug, Clone)]
pub struct AppState {
    pub loaded_geohashes: usize,
    pub pending_api_requests: usize,
    pub results: Vec<QueryResult>,
    pub taxa_store: TaxaStore,
    pub current_observation_id: Option<ObservationId>,
    pub current_view: AppView,
    pub is_authenticated: bool,
    pub show_login_modal: bool,
    pub auth_status_message: Option<String>,
    pub current_user: Option<User>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            loaded_geohashes: 0,
            pending_api_requests: 0,
            results: Vec::new(),
            taxa_store: TaxaStore::default(),
            current_observation_id: None,
            current_view: AppView::Identify,
            is_authenticated: false,
            show_login_modal: false,
            auth_status_message: None,
            current_user: None,
        }
    }
}

// --- AppMessage ---
#[derive(Debug)]
pub enum AppMessage {
    Progress,
    TaxonLoaded(Box<ShowTaxon>),
    SkipCurrentObservation,
    ObservationLoaded(Box<Observation>),
    ComputerVisionScoreLoaded(
        ObservationId,
        Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    ),
    TaxonTree {
        observation_id: i32,
        taxon_tree: taxon_tree::TaxonTree,
    },
    AuthenticationCodeReceived(AuthorizationCode),
    Authenticated(String),
    AuthError(String),
    InitiateLogin,
    StartLoadingObservations,
    UserLoaded(User),
    PendingRequestsCount(usize),
}
