use actix::prelude::*;
use inaturalist_pro_core::AppMessage;
use tokio::sync::mpsc::UnboundedSender;

pub struct UserLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub api_token: String,
}

impl Actor for UserLoaderActor {
    type Context = Context<Self>;
}

pub struct FetchCurrentUserMessage;

impl Message for FetchCurrentUserMessage {
    type Result = ();
}

impl Handler<FetchCurrentUserMessage> for UserLoaderActor {
    type Result = ();

    fn handle(&mut self, _msg: FetchCurrentUserMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();
        let api_token = self.api_token.clone();

        ctx.wait(
            Box::pin(async move {
                match inaturalist_fetch::fetch_current_user(&api_token).await {
                    Ok(user) => {
                        let _ = tx_app_message.send(AppMessage::UserLoaded(user));
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch user info: {:?}", e);
                    }
                }
            })
            .into_actor(self),
        );
    }
}
