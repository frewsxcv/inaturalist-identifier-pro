use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct TaxonTree(pub HashMap<i32, TaxonTreeNode>);

#[derive(Debug, Default, Clone)]
pub struct TaxonTreeNode {
    // pub taxon: ShowTaxon,
    pub taxon_id: i32,
    pub children: TaxonTree,
    pub score: f32,
}
