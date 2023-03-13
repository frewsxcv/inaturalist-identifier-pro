use crate::taxon_tree::TaxonTreeNode;

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
        todo!()
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
            let mut hash_map = <crate::taxon_tree::TaxonTree as Default>::default();
            for (taxon_guess, score) in taxa.iter().zip(msg.scores.iter().map(|s| s.combined_score))
            {
                let mut foo = &mut hash_map;
                for ancestor_id in taxon_guess.ancestor_ids.as_ref().unwrap() {
                    // let inner_result = inaturalist_fetch::fetch_taxa(vec![*ancestor_id])
                    // .await
                    // .unwrap();
                    let taxon_tree_node = foo
                        .0
                        .entry(*ancestor_id)
                        .and_modify(|n| n.score += score)
                        .or_insert_with(|| TaxonTreeNode {
                            taxon_id: *ancestor_id,
                            children: Default::default(),
                            score,
                        });
                    foo = &mut taxon_tree_node.children;
                }
                foo.0
                    .entry(taxon_guess.id.unwrap())
                    .and_modify(|n| n.score += score)
                    .or_insert_with(|| TaxonTreeNode {
                        taxon_id: taxon_guess.id.unwrap(),
                        children: Default::default(),
                        score,
                    });
            }
            tx_app_message
                .send(crate::AppMessage::TaxonTree {
                    observation_id: msg.observation_id,
                    taxon_tree: hash_map,
                })
                .unwrap();
        };

        ctx.spawn(Box::pin(t).into_actor(self));
    }
}
