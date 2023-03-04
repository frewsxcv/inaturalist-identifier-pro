use crate::{operations::Operation, AppMessage};
use actix::prelude::*;
use inaturalist::models::Observation;
use tokio::sync::mpsc::UnboundedSender;

pub struct ObservationProcessorActor {
    pub operation: crate::CurOperation,
    pub tx_app_message: UnboundedSender<AppMessage>,
}

impl Default for ObservationProcessorActor {
    fn default() -> Self {
        unimplemented!()
    }
}

pub struct ProcessObservationMessage {
    observation: Observation,
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

    fn handle(&mut self, msg: ProcessObservationMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();
        self.operation
            .visit_observation(msg.observation, tx_app_message);
    }
}
