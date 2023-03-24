use crate::AppMessage;
use tokio::sync::mpsc::UnboundedSender;
use actix::prelude::*;

type TaxaId = i32;

pub struct TaxaLoaderActor {
    pub tx_app_message: UnboundedSender<AppMessage>,
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

impl Handler<LoadTaxaMessage> for TaxaLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: LoadTaxaMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();
        ctx.spawn(
            Box::pin(async move {
                /*
                {
                    let mut lock = TAXA_STORE.write().await;

                    for taxa_id in msg.0 {
                        lock.0.entry(taxa_id).or_insert(crate::taxa_store::TaxaValue::Loading);
                    }
                }
                */

                /*
                let taxa_ids = TAXA_STORE
                    .read()
                    .await
                    .0
                    .iter()
                    .filter(|(_, v)| matches!(v, TaxaValue::Loading))
                    .map(|(k, _)| k)
                    .copied()
                    .collect::<Vec<_>>();
                */

                let taxa = inaturalist_fetch::fetch_taxa(msg.0)
                    .await
                    .unwrap();

                for taxon in taxa.results {
                    tx_app_message.send(
                        AppMessage::TaxonLoaded(Box::new(taxon))
                    ).unwrap();
                }
            })
            .into_actor(self),
        );
    }
}

/*
struct LoadTaxonMessage(TaxaId);

impl Message for LoadTaxonMessage {
    type Result = ();
}

impl Handler<LoadTaxonMessage> for TaxaLoaderActor {
    type Result = ();

    fn handle(&mut self, msg: LoadTaxonMessage, ctx: &mut Self::Context) -> Self::Result {
    }
}

fn load(ctx: &mut <TaxaLoaderActor as Actor>::Context) {
        ctx.spawn(
            Box::pin(async move {
                {
                    QUEUE.write().await.insert(msg.0);
                }

                let taxa_ids = QUEUE.read().await.iter().copied().collect::<Vec<i32>>();

                let taxa = inaturalist_fetch::fetch_taxa(taxa_ids.clone())
                    .await
                    .unwrap();

                {
                    let mut lock = QUEUE.write().await;
                    for taxa_id in &taxa_ids {
                        lock.remove(taxa_id);
                    }
                }

                {
                    let mut lock = crate::taxa_store::TAXA_STORE.write().await;
                    for taxon in taxa.results {
                        lock.0.insert(
                            taxon.id.unwrap(),
                            crate::taxa_store::Taxon {
                                id: taxon.id.unwrap(),
                                name: taxon.preferred_common_name.unwrap(),
                            },
                        );
                    }
                }
            })
            .into_actor(self),
        );

}
*/
