use crate::AppMessage;
use actix::prelude::*;
use std::collections;
use tokio::sync::mpsc::UnboundedSender;

type TaxaId = i32;

pub struct TaxaLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
    pub seen: collections::HashSet<TaxaId>,
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

pub struct LoadTaxaMessage(pub Vec<TaxaId>);

impl Message for LoadTaxaMessage {
    type Result = ();
}

pub struct TaxonLoadedMessage(pub TaxaId);

impl Message for TaxonLoadedMessage {
    type Result = ();
}

impl Handler<TaxonLoadedMessage> for TaxaLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: TaxonLoadedMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.seen.insert(msg.0);
    }
}

impl Handler<LoadTaxaMessage> for TaxaLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: LoadTaxaMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();

        let taxa_id_to_fetch = msg
            .0
            .into_iter()
            .filter(|id| !self.seen.contains(id))
            .collect::<Vec<i32>>();

        if taxa_id_to_fetch.is_empty() {
            return;
        }

        ctx.spawn(
            Box::pin(async move {
                let taxa = inaturalist_fetch::fetch_taxa(taxa_id_to_fetch).await.unwrap();

                for taxon in taxa.results {
                    TaxaLoaderActor::from_registry()
                        .try_send(TaxonLoadedMessage(taxon.id.unwrap()))
                        .unwrap();

                    tx_app_message
                        .send(AppMessage::TaxonLoaded(Box::new(taxon)))
                        .unwrap();
                }
            })
            .into_actor(self),
        );
    }
}
