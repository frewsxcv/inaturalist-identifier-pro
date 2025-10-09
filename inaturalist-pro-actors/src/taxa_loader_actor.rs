use actix::prelude::*;
use inaturalist_pro_core::AppMessage;
use std::collections;
use tokio::sync::mpsc::UnboundedSender;

type TaxonId = i32;

pub struct TaxaLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub to_load: collections::HashSet<TaxonId>,
    pub loaded: collections::HashSet<TaxonId>,
    pub api_token: String,
}

impl Default for TaxaLoaderActor {
    fn default() -> Self {
        unimplemented!()
    }
}

impl SystemService for TaxaLoaderActor {}

impl Supervised for TaxaLoaderActor {}

impl Actor for TaxaLoaderActor {
    type Context = Context<Self>;
}

pub struct LoadTaxonMessage(pub TaxonId);

impl Message for LoadTaxonMessage {
    type Result = ();
}

pub struct FetchTaxaMessage;

impl Message for FetchTaxaMessage {
    type Result = ();
}

impl Handler<FetchTaxaMessage> for TaxaLoaderActor {
    type Result = ();

    fn handle(&mut self, _msg: FetchTaxaMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();

        let taxa_ids_to_fetch = self
            .to_load
            .difference(&self.loaded)
            .copied()
            .take(30) // Maxmimum number allowed
            .collect::<Vec<_>>();

        if taxa_ids_to_fetch.is_empty() {
            return;
        }

        ctx.wait(
            Box::pin({
                let taxa_ids_to_fetch = taxa_ids_to_fetch.clone();
                let api_token = self.api_token.clone();
                async move {
                    let taxa = inaturalist_fetch::fetch_taxa(taxa_ids_to_fetch, &api_token)
                        .await
                        .unwrap();

                    for taxon in taxa.results {
                        tx_app_message
                            .send(AppMessage::TaxonLoaded(Box::new(taxon)))
                            .unwrap();
                        // TODO: send new message to self (blocking) that removes the entries from the hashsets
                    }
                }
            })
            .into_actor(self),
        );

        // TODO: clear to_load?
        for taxon_id in taxa_ids_to_fetch {
            self.loaded.insert(taxon_id);
        }
    }
}

impl Handler<LoadTaxonMessage> for TaxaLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: LoadTaxonMessage, ctx: &mut Self::Context) -> Self::Result {
        self.to_load.insert(msg.0);
        ctx.notify(FetchTaxaMessage);
    }
}
