use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TaxonTree(pub HashMap<i32, TaxonTreeNode>);

#[derive(Debug, Default)]
pub struct TaxonTreeNode {
    pub children: TaxonTree,
    pub score: Option<f32>,
}
