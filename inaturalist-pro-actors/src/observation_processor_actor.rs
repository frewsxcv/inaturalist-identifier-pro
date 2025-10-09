use actix::prelude::*;
use inaturalist::models::Observation;
use inaturalist_pro_core::AppMessage;
use tokio::sync::mpsc::UnboundedSender;

pub struct ObservationProcessorActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub api_token: String,
    pub active_requests: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl Default for ObservationProcessorActor {
    fn default() -> Self {
        unimplemented!()
    }
}

pub struct ProcessObservationMessage {
    pub observation: Observation,
}

impl Message for ProcessObservationMessage {
    type Result = ();
}

impl actix::Supervised for ObservationProcessorActor {}

impl SystemService for ObservationProcessorActor {}

impl Actor for ObservationProcessorActor {
    type Context = Context<Self>;
}

impl Handler<ProcessObservationMessage> for ObservationProcessorActor {
    type Result = ();

    fn handle(&mut self, msg: ProcessObservationMessage, _ctx: &mut Self::Context) -> Self::Result {
        tracing::info!("Processing observation: {:?}", msg.observation.id);

        let observation_id = msg.observation.id.unwrap();
        let tx_app_message = self.tx_app_message.clone();
        let api_token = self.api_token.clone();
        let observation = msg.observation;
        let active_requests = self.active_requests.clone();

        // Send the observation loaded message
        let _ = tx_app_message.send(AppMessage::ObservationLoaded(Box::new(observation.clone())));

        // Increment active requests counter for CV fetch
        active_requests.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Spawn task to fetch computer vision scores
        actix::spawn(async move {
            let results = inaturalist_fetch::fetch_computer_vision_observation_scores(
                &observation,
                &api_token,
            )
            .await;

            match results {
                Ok(scores) => {
                    let _ = tx_app_message.send(AppMessage::ComputerVisionScoreLoaded(
                        observation_id,
                        scores.results,
                    ));
                }
                Err(inaturalist_fetch::FetchComputerVisionError::Unauthorized) => {
                    tracing::error!(
                        "API token expired. Could not fetch computer vision scores for observation {}",
                        observation_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Could not fetch computer vision scores for observation {}: {}",
                        observation_id,
                        e
                    );
                }
            }

            // Decrement active requests counter
            active_requests.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
    }
}
