use crate::AppMessage;
use actix::prelude::*;
use inaturalist_oauth::{Authenticator, PkceVerifier};
use oauth2::AuthorizationCode;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ExchangeCode {
    pub code: AuthorizationCode,
    pub client_id: String,
    pub client_secret: String,
    pub pkce_verifier: PkceVerifier,
}

pub struct OauthActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
}

impl OauthActor {
    pub fn new(tx_app_message: UnboundedSender<AppMessage>) -> Self {
        Self { tx_app_message }
    }
}

impl Actor for OauthActor {
    type Context = Context<Self>;
}

impl Handler<ExchangeCode> for OauthActor {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ExchangeCode, _ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();

        Box::pin(async move {
            let authenticator = Authenticator::new(msg.client_id, msg.client_secret);
            match authenticator
                .exchange_code(msg.code, msg.pkce_verifier)
                .await
            {
                Ok(token_details) => {
                    tracing::info!("Authentication successful!");
                    tracing::info!("Received API token: {}", token_details.api_token);

                    // Save token
                    let cfg = crate::MyConfig {
                        token: Some(token_details.clone()),
                    };
                    if let Err(e) = confy::store("inaturalist-identifier-pro", None, cfg) {
                        tracing::error!("Failed to save token: {}", e);
                    } else {
                        tracing::info!("Token saved successfully");
                    }

                    let _ = tx_app_message.send(AppMessage::Authenticated(token_details.api_token));
                }
                Err(e) => {
                    tracing::error!("Authentication failed with error: {}", e);
                    let _ = tx_app_message.send(AppMessage::AuthError(format!(
                        "Authentication failed: {}. Please try again.",
                        e
                    )));
                }
            }
        })
    }
}
