use crate::AppMessage;
use actix::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

pub struct IdentifyActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
}

impl Default for IdentifyActor {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SystemService for IdentifyActor {}

impl Supervised for IdentifyActor {}

impl Actor for IdentifyActor {
    type Context = Context<Self>;
}

pub struct IdentifyMessage {
    pub observation_id: i32,
    pub taxon_id: i32,
}

impl Message for IdentifyMessage {
    type Result = ();
}

impl Handler<IdentifyMessage> for IdentifyActor {
    type Result = ();

    fn handle(&mut self, msg: IdentifyMessage, ctx: &mut Self::Context) -> Self::Result {
        // let tx_app_message = self.tx_app_message.clone();
        ctx.spawn(
            Box::pin(async move {
                tracing::info!(
                    "Identifying observation ID={} with taxon ID={}",
                    msg.observation_id,
                    msg.taxon_id
                );
                if let Err(e) = inaturalist_fetch::identify(msg.observation_id, msg.taxon_id).await
                {
                    tracing::error!("Encountered an error while identifying: {:?}", e);
                }
            })
            .into_actor(self),
        );
    }
}
