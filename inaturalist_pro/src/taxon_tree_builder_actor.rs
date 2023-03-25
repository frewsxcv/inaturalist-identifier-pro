use crate::{
    taxa_loader_actor::{TaxaLoaderActor, LoadTaxonMessage},
    taxon_tree::TaxonTreeNode,
};

use actix::prelude::*;

pub struct TaxonTreeBuilderActor {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<crate::AppMessage>,
}

pub struct BuildTaxonTreeMessage {
    pub scores: Vec<inaturalist_fetch::ComputerVisionObservationScore>,
    pub observation_id: i32,
}

impl Message for BuildTaxonTreeMessage {
    type Result = ();
}

impl Default for TaxonTreeBuilderActor {
    fn default() -> Self {
        unimplemented!()
    }
}

impl actix::Supervised for TaxonTreeBuilderActor {}

impl SystemService for TaxonTreeBuilderActor {}

impl Actor for TaxonTreeBuilderActor {
    type Context = Context<Self>;
}

impl Handler<BuildTaxonTreeMessage> for TaxonTreeBuilderActor {
    type Result = ();

    fn handle(&mut self, msg: BuildTaxonTreeMessage, ctx: &mut Self::Context) -> Self::Result {
        let tx_app_message = self.tx_app_message.clone();
        let t = async move {
            let taxa_ids = msg
                .scores
                .iter()
                .map(|n| n.taxon.id.unwrap())
                .collect::<Vec<_>>();
            let taxa = inaturalist_fetch::fetch_taxa(taxa_ids)
                .await
                .unwrap()
                .results;
            let mut taxon_tree = <crate::taxon_tree::TaxonTree as Default>::default();
            for (taxon_guess, score) in taxa.iter().zip(msg.scores.iter().map(|s| s.combined_score))
            {
                tx_app_message
                    .send(crate::AppMessage::TaxonLoaded(Box::new(
                        taxon_guess.clone(),
                    )))
                    .unwrap();

                let mut curr_taxon_tree = &mut taxon_tree;
                for ancestor_taxon_id in taxon_guess
                    .ancestor_ids
                    .as_ref()
                    .unwrap()
                    .iter()
                    .chain(std::iter::once(&taxon_guess.id.unwrap()))
                {
                    if let Some(index) = curr_taxon_tree
                        .0
                        .iter()
                        .position(|n| n.taxon_id == *ancestor_taxon_id)
                    {
                        curr_taxon_tree.0[index].score += score;
                        curr_taxon_tree = &mut curr_taxon_tree.0[index].children;
                    } else {
                        curr_taxon_tree.0.push(TaxonTreeNode {
                            taxon_id: *ancestor_taxon_id,
                            children: Default::default(),
                            score,
                        });
                        curr_taxon_tree = &mut curr_taxon_tree.0.last_mut().unwrap().children;
                    }
                    curr_taxon_tree
                        .0
                        .sort_by(|n, m| n.score.partial_cmp(&m.score).unwrap().reverse());
                    TaxaLoaderActor::from_registry()
                        .try_send(LoadTaxonMessage(*ancestor_taxon_id))
                        .unwrap();
                }
            }
            tx_app_message
                .send(crate::AppMessage::TaxonTree {
                    observation_id: msg.observation_id,
                    taxon_tree,
                })
                .unwrap();
        };

        ctx.spawn(Box::pin(t).into_actor(self));
    }
}
