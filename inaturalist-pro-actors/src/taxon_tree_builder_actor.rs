use crate::taxa_loader_actor::{LoadTaxonMessage, TaxaLoaderActor};

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
            let taxa_ids: Vec<_> = msg.scores.iter().filter_map(|n| n.taxon.id).collect();

            if taxa_ids.is_empty() {
                tracing::warn!("No valid taxon IDs in computer vision scores");
                return;
            }

            let taxa = match inaturalist_fetch::fetch_taxa(taxa_ids, &api_token).await {
                Ok(result) => result.results,
                Err(e) => {
                    tracing::error!("Failed to fetch taxa: {}", e);
                    return;
                }
            };

            let taxa_map: collections::HashMap<TaxaId, inaturalist::models::ShowTaxon> = taxa
                .into_iter()
                .filter_map(|t| t.id.map(|id| (id, t)))
                .collect();

            let mut taxon_tree = TaxonTree::default();

            for (taxon_guess, _score_value) in taxa_map
                .values()
                .zip(msg.scores.iter().map(|s| s.combined_score))
            {
                if let Err(e) =
                    tx_app_message.send(AppMessage::TaxonLoaded(Box::new(taxon_guess.clone())))
                {
                    tracing::error!("Failed to send TaxonLoaded message: {}", e);
                    continue;
                }

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

                    // Skip if we don't have this taxon in our map (ancestors weren't fetched)
                    let Some(taxon_from_map) = taxon_from_map_opt else {
                        tracing::warn!("Taxon {} not found in fetched taxa, skipping", taxon_id);
                        continue;
                    };

                    let taxon = Taxon::from(taxon_from_map);

                    // Get or insert the current node
                    let node_entry = taxon_tree.nodes.entry(taxon_id);
                    let _node = node_entry.or_insert_with(|| TaxonNode {
                        taxon,
                        children: Vec::new(),
                    });

                    // Add this node to its parent's children list
                    if let Some(parent_id) = current_parent_id {
                        if let Some(parent_node) = taxon_tree.nodes.get_mut(&parent_id) {
                            if !parent_node.children.contains(&taxon_id) {
                                parent_node.children.push(taxon_id);
                            }
                        }
                    }

                    current_parent_id = Some(taxon_id);

                    if let Err(e) =
                        TaxaLoaderActor::from_registry().try_send(LoadTaxonMessage(taxon_id))
                    {
                        tracing::warn!("Failed to send LoadTaxonMessage: {}", e);
                    }
                }
            }
            if let Err(e) = tx_app_message.send(AppMessage::TaxonTree {
                observation_id: msg.observation_id,
                taxon_tree,
            }) {
                tracing::error!("Failed to send TaxonTree message: {}", e);
            }
        };

        ctx.spawn(Box::pin(t).into_actor(self));
    }
}
