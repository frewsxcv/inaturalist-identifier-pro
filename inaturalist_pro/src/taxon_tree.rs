#[derive(Debug, Default, Clone)]
pub struct TaxonTree(pub Vec<TaxonTreeNode>);

#[derive(Debug, Default, Clone)]
pub struct TaxonTreeNode {
    // pub taxon: ShowTaxon,
    pub taxon_id: i32,
    pub children: TaxonTree,
    pub score: f32,
}
