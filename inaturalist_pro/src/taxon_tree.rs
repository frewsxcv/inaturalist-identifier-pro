use std::collections::HashMap;

use inaturalist::models::ShowTaxon;

#[derive(Debug, Default, Clone)]
pub struct TaxonTree(pub HashMap<i32, TaxonTreeNode>);

#[derive(Debug, Default, Clone)]
pub struct TaxonTreeNode {
    pub taxon: ShowTaxon,
    pub children: TaxonTree,
    pub score: Option<f32>,
}
