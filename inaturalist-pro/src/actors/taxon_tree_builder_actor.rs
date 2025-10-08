use crate::actors::taxa_loader_actor::{LoadTaxonMessage, TaxaLoaderActor};

use actix::prelude::*;

use inaturalist_pro_core::{
    taxon_tree::{TaxonNode, TaxonTree},
    AppMessage, TaxaId, Taxon,
};
use std::collections;

pub struct TaxonTreeBuilderActor {
    pub tx_app_message: tokio::sync::mpsc::UnboundedSender<AppMessage>,
    pub api_token: String,
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
        let api_token = self.api_token.clone();
        let t = async move {
            let taxa_ids = msg
                .scores
                .iter()
                .map(|n| n.taxon.id.unwrap())
                .collect::<Vec<_>>();
            let taxa = inaturalist_fetch::fetch_taxa(taxa_ids, &api_token)
                .await
                .unwrap()
                .results;
            let taxa_map: collections::HashMap<TaxaId, inaturalist::models::ShowTaxon> =
                taxa.into_iter().map(|t| (t.id.unwrap(), t)).collect();

            let mut taxon_tree = TaxonTree::default();

            for (taxon_guess, score_value) in taxa_map
                .values()
                .zip(msg.scores.iter().map(|s| s.combined_score))
            {
                tx_app_message
                    .send(AppMessage::TaxonLoaded(Box::new(taxon_guess.clone())))
                    .unwrap();

                let mut current_parent_id: Option<TaxaId> = None;

                let mut full_ancestry_ids: Vec<TaxaId> = taxon_guess
                    .ancestor_ids
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .copied()
                    .collect();
                full_ancestry_ids.push(taxon_guess.id.unwrap());

                for &taxon_id in full_ancestry_ids.iter() {
                    let taxon_from_map_opt = taxa_map.get(&taxon_id);
                    let taxon = Taxon::from(taxon_from_map_opt.unwrap());

                    // Get or insert the current node
                    let node_entry = taxon_tree.nodes.entry(taxon_id);
                    let node = node_entry.or_insert_with(|| TaxonNode {
                        taxon,
                        children: Vec::new(),
                    });

                    // Add this node to its parent's children list
                    if let Some(parent_id) = current_parent_id {
                        let parent_node = taxon_tree.nodes.get_mut(&parent_id).unwrap();
                        if !parent_node.children.contains(&taxon_id) {
                            parent_node.children.push(taxon_id);
                        }
                    }

                    current_parent_id = Some(taxon_id);

                    TaxaLoaderActor::from_registry()
                        .try_send(LoadTaxonMessage(taxon_id))
                        .unwrap();
                }
            }
            tx_app_message
                .send(AppMessage::TaxonTree {
                    observation_id: msg.observation_id,
                    taxon_tree,
                })
                .unwrap();
        };

        ctx.spawn(Box::pin(t).into_actor(self));
    }
}
