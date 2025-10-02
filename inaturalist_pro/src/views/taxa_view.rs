use egui::{RichText, ScrollArea};

#[derive(Default)]
pub struct TaxaView {
    search_query: String,
    search_results: Vec<TaxonSearchResult>,
    selected_taxon: Option<TaxonDetails>,
    loading: bool,
    rank_filter: TaxonRank,
}

#[derive(Debug, Clone, PartialEq)]
enum TaxonRank {
    Any,
    Kingdom,
    Phylum,
    Class,
    Order,
    Family,
    Genus,
    Species,
}

impl Default for TaxonRank {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, Clone)]
struct TaxonSearchResult {
    id: i32,
    name: String,
    common_name: Option<String>,
    rank: String,
    observations_count: i32,
}

#[derive(Debug, Clone)]
struct TaxonDetails {
    id: i32,
    name: String,
    common_name: Option<String>,
    rank: String,
    observations_count: i32,
    wikipedia_summary: Option<String>,
    ancestry: Vec<AncestorTaxon>,
    iconic_taxon_name: Option<String>,
}

#[derive(Debug, Clone)]
struct AncestorTaxon {
    name: String,
    rank: String,
}

impl TaxaView {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("taxa_search_panel")
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.render_search_panel(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                self.render_taxon_details(ui);
            });
        });
    }

    fn render_search_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌿 Search Taxa");
        ui.separator();

        // Search form
        ui.label("Taxon name:");
        ui.text_edit_singleline(&mut self.search_query);

        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Rank:");
            egui::ComboBox::from_id_salt("rank_filter_combo")
                .selected_text(format!("{:?}", self.rank_filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Any, "Any");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Kingdom, "Kingdom");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Phylum, "Phylum");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Class, "Class");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Order, "Order");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Family, "Family");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Genus, "Genus");
                    ui.selectable_value(&mut self.rank_filter, TaxonRank::Species, "Species");
                });
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("🔍 Search").clicked() {
                self.perform_search();
            }

            if ui.button("Clear").clicked() {
                self.clear();
            }
        });

        ui.add_space(15.0);
        ui.separator();

        // Search results
        if self.loading {
            ui.spinner();
            ui.label("Searching...");
        } else if self.search_results.is_empty() {
            ui.label(RichText::new("No results").italics().weak());
        } else {
            ui.label(RichText::new(format!("{} results", self.search_results.len())).strong());
            ui.add_space(10.0);

            let mut clicked_taxon_id = None;
            for result in &self.search_results {
                let response = ui.add(
                    egui::Button::new(self.format_taxon_result(result))
                        .wrap()
                        .fill(egui::Color32::TRANSPARENT),
                );

                if response.clicked() {
                    clicked_taxon_id = Some(result.id);
                }

                ui.add_space(5.0);
            }

            if let Some(taxon_id) = clicked_taxon_id {
                self.select_taxon(taxon_id);
            }
        }
    }

    fn render_taxon_details(&mut self, ui: &mut egui::Ui) {
        let Some(taxon) = &self.selected_taxon else {
            ui.heading("Taxa Explorer");
            ui.separator();
            ui.add_space(20.0);
            ui.label(
                RichText::new("Search for a taxon in the left panel to view its details")
                    .italics()
                    .weak(),
            );
            ui.add_space(20.0);
            ui.label("🌿 Explore the tree of life");
            ui.label("🔍 Search for any organism");
            ui.label("📊 View observation counts");
            ui.label("📖 Read taxonomic information");
            return;
        };

        // Taxon header
        ui.horizontal(|ui| {
            ui.heading(&taxon.name);
            ui.label(RichText::new(&taxon.rank).weak());
        });

        if let Some(common_name) = &taxon.common_name {
            ui.label(RichText::new(common_name).size(16.0));
        }

        ui.label(
            RichText::new(format!("Taxon ID: {}", taxon.id))
                .small()
                .weak(),
        );

        ui.add_space(15.0);
        ui.separator();

        // Observation count
        ui.horizontal(|ui| {
            ui.label("📷 Observations:");
            ui.label(RichText::new(format!("{}", taxon.observations_count)).strong());
        });

        if let Some(iconic_taxon) = &taxon.iconic_taxon_name {
            ui.horizontal(|ui| {
                ui.label("🏷️ Iconic Taxon:");
                ui.label(iconic_taxon);
            });
        }

        ui.add_space(15.0);
        ui.separator();

        // Taxonomy
        if !taxon.ancestry.is_empty() {
            ui.heading("Taxonomy");
            ui.add_space(10.0);

            for (i, ancestor) in taxon.ancestry.iter().enumerate() {
                let indent = "  ".repeat(i);
                ui.horizontal(|ui| {
                    ui.label(format!("{}{}", indent, ancestor.rank));
                    ui.label(RichText::new(&ancestor.name).strong());
                });
            }

            // Add current taxon at the end
            let indent = "  ".repeat(taxon.ancestry.len());
            ui.horizontal(|ui| {
                ui.label(format!("{}{}", indent, taxon.rank));
                ui.label(
                    RichText::new(&taxon.name)
                        .strong()
                        .color(egui::Color32::GREEN),
                );
            });

            ui.add_space(15.0);
            ui.separator();
        }

        // Wikipedia summary
        if let Some(summary) = &taxon.wikipedia_summary {
            ui.heading("About");
            ui.add_space(10.0);
            ui.label(summary);
        }
    }

    fn format_taxon_result(&self, result: &TaxonSearchResult) -> String {
        let mut text = format!("{}", result.name);
        if let Some(common_name) = &result.common_name {
            text.push_str(&format!("\n{}", common_name));
        }
        text.push_str(&format!(
            "\n{} • {} obs",
            result.rank, result.observations_count
        ));
        text
    }

    fn perform_search(&mut self) {
        // TODO: Implement actual API search
        self.loading = false;
        // This is a placeholder - in a real implementation, this would:
        // 1. Build query parameters from the search query and filters
        // 2. Send a message to an actor to fetch taxa
        // 3. Update self.search_results when the data arrives

        // For now, show example data when searching
        if !self.search_query.is_empty() {
            self.search_results = vec![
                TaxonSearchResult {
                    id: 1,
                    name: "Animalia".to_string(),
                    common_name: Some("Animals".to_string()),
                    rank: "Kingdom".to_string(),
                    observations_count: 50000000,
                },
                TaxonSearchResult {
                    id: 47126,
                    name: "Plantae".to_string(),
                    common_name: Some("Plants".to_string()),
                    rank: "Kingdom".to_string(),
                    observations_count: 30000000,
                },
                TaxonSearchResult {
                    id: 47170,
                    name: "Fungi".to_string(),
                    common_name: Some("Fungi".to_string()),
                    rank: "Kingdom".to_string(),
                    observations_count: 5000000,
                },
            ];
        }
    }

    fn select_taxon(&mut self, taxon_id: i32) {
        // TODO: Implement actual API fetch for taxon details
        // For now, create example data
        self.selected_taxon = Some(TaxonDetails {
            id: taxon_id,
            name: "Animalia".to_string(),
            common_name: Some("Animals".to_string()),
            rank: "Kingdom".to_string(),
            observations_count: 50000000,
            iconic_taxon_name: Some("Animalia".to_string()),
            wikipedia_summary: Some(
                "Animals are multicellular eukaryotic organisms that form the biological kingdom Animalia. With few exceptions, animals consume organic material, breathe oxygen, are able to move, can reproduce sexually, and grow from a hollow sphere of cells, the blastula, during embryonic development.".to_string()
            ),
            ancestry: vec![],
        });
    }

    fn clear(&mut self) {
        self.search_query.clear();
        self.search_results.clear();
        self.rank_filter = TaxonRank::Any;
    }
}
