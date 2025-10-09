use inaturalist::models::Observation;
use inaturalist_pro_core::{TaxaStore, taxon_tree::TaxonNode};

pub struct TaxonTreeWidget<'a> {
    pub observation: &'a Observation,
    pub root_node: &'a TaxonNode,
    pub taxa_store: &'a TaxaStore,
    pub identified: &'a mut bool,
}

impl<'a> egui::Widget for TaxonTreeWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let collapsing_header_id = format!(
            "{}-{}",
            self.observation.id.unwrap(),
            self.root_node.taxon.id,
        );

        let (response, _, _) = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            collapsing_header_id.into(),
            true,
        )
        .show_header(ui, |ui| {
            ui.horizontal(|ui| {
                // Display taxon name
                let taxon_name = &self.root_node.taxon.name;

                if ui.button(taxon_name).clicked() {
                    *self.identified = true;
                    // TODO: Send identification to iNaturalist
                }
            })
            .response
        })
        .body(|ui| {
            // Recursively show children
            for child_id in &self.root_node.children {
                if let Some(child_taxon) = self.taxa_store.0.get(child_id) {
                    ui.label(format!("  → {}", child_taxon.name));
                }
            }
        });

        response
    }
}
