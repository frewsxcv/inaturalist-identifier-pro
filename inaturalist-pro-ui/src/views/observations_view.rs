use egui::{RichText, ScrollArea};

#[derive(Default)]
pub struct ObservationsView {
    search_query: String,
    taxon_filter: String,
    user_filter: String,
    place_filter: String,
    date_from: String,
    date_to: String,
    quality_grade: QualityGrade,
    identified: IdentifiedFilter,
    results: Vec<ObservationResult>,
    loading: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum QualityGrade {
    Any,
    Research,
    NeedsId,
    Casual,
}

impl Default for QualityGrade {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, Clone, PartialEq)]
enum IdentifiedFilter {
    Any,
    Yes,
    No,
}

impl Default for IdentifiedFilter {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Debug, Clone)]
struct ObservationResult {
    id: i32,
    taxon_name: String,
    user_login: String,
    observed_on: String,
    place: String,
}

impl ObservationsView {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔍 Query Observations");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                self.render_search_form(ui);
                ui.add_space(20.0);
                self.render_results(ui);
            });
        });
    }

    fn render_search_form(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("observation_search_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                // General search
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
                ui.end_row();

                // Taxon filter
                ui.label("Taxon:");
                ui.text_edit_singleline(&mut self.taxon_filter);
                ui.end_row();

                // User filter
                ui.label("User:");
                ui.text_edit_singleline(&mut self.user_filter);
                ui.end_row();

                // Place filter
                ui.label("Place:");
                ui.text_edit_singleline(&mut self.place_filter);
                ui.end_row();

                // Date range
                ui.label("Date from:");
                ui.text_edit_singleline(&mut self.date_from);
                ui.end_row();

                ui.label("Date to:");
                ui.text_edit_singleline(&mut self.date_to);
                ui.end_row();

                // Quality grade
                ui.label("Quality:");
                egui::ComboBox::from_id_salt("quality_grade_combo")
                    .selected_text(format!("{:?}", self.quality_grade))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.quality_grade, QualityGrade::Any, "Any");
                        ui.selectable_value(
                            &mut self.quality_grade,
                            QualityGrade::Research,
                            "Research",
                        );
                        ui.selectable_value(
                            &mut self.quality_grade,
                            QualityGrade::NeedsId,
                            "Needs ID ",
                        );
                        ui.selectable_value(
                            &mut self.quality_grade,
                            QualityGrade::Casual,
                            "Casual",
                        );
                    });
                ui.end_row();

                // Identified filter
                ui.label("Identified:");
                egui::ComboBox::from_id_salt("identified_combo")
                    .selected_text(format!("{:?}", self.identified))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.identified, IdentifiedFilter::Any, "Any");
                        ui.selectable_value(&mut self.identified, IdentifiedFilter::Yes, "Yes");
                        ui.selectable_value(&mut self.identified, IdentifiedFilter::No, "No");
                    });
                ui.end_row();
            });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("🔍 Search ").clicked() {
                self.perform_search();
            }

            if ui.button("Clear").clicked() {
                self.clear_filters();
            }
        });
    }

    fn render_results(&mut self, ui: &mut egui::Ui) {
        if self.loading {
            ui.spinner();
            ui.label("Loading observations...");
            return;
        }

        if self.results.is_empty() {
            ui.label(
                RichText::new(
                    "No observations to display. Use the search form above to query observations.",
                )
                .italics()
                .weak(),
            );
            return;
        }

        ui.heading(format!("Results ({})", self.results.len()));
        ui.separator();

        egui::Grid::new("observation_results_grid")
            .num_columns(5)
            .striped(true)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                // Header
                ui.strong("ID");
                ui.strong("Taxon");
                ui.strong("User");
                ui.strong("Date");
                ui.strong("Place");
                ui.end_row();

                // Results
                for result in &self.results {
                    ui.label(format!("{}", result.id));
                    ui.label(&result.taxon_name);
                    ui.label(&result.user_login);
                    ui.label(&result.observed_on);
                    ui.label(&result.place);
                    ui.end_row();
                }
            });
    }

    fn perform_search(&mut self) {
        // TODO: Implement actual API search
        self.loading = false;
        // This is a placeholder - in a real implementation, this would:
        // 1. Build query parameters from the filters
        // 2. Send a message to an actor to fetch observations
        // 3. Update self.results when the data arrives
    }

    fn clear_filters(&mut self) {
        self.search_query.clear();
        self.taxon_filter.clear();
        self.user_filter.clear();
        self.place_filter.clear();
        self.date_from.clear();
        self.date_to.clear();
        self.quality_grade = QualityGrade::Any;
        self.identified = IdentifiedFilter::Any;
        self.results.clear();
    }
}
