use crate::movie::Movie;
use eframe::egui::{self, CursorIcon, Margin};
use egui::{Color32, CornerRadius, RichText, Stroke, Vec2};
use std::{fs::File, path::Path};

struct ColorTheme {
    primary: Color32,
    primary_light: Color32,
    primary_dark: Color32,
    secondary: Color32,
    background: Color32,
    card_bg: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    border_light: Color32,
    selected_bg: Color32,
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme {
            primary: Color32::from_rgb(210, 144, 84),
            primary_light: Color32::from_rgb(237, 184, 121),
            primary_dark: Color32::from_rgb(160, 95, 50),
            secondary: Color32::from_rgb(235, 235, 235),
            background: Color32::from_rgb(24, 24, 24),
            card_bg: Color32::from_rgb(36, 36, 36),
            text_primary: Color32::from_rgb(235, 235, 235),
            text_secondary: Color32::from_rgb(160, 160, 160),
            border_light: Color32::from_rgb(64, 64, 64),
            selected_bg: Color32::from_rgb(54, 45, 38),
        }
    }
}

const TOP_N: usize = 10;

#[derive(Default)]
pub struct MovieSimilarityApp {
    movies: Vec<Movie>,
    selected_movie_index: Option<usize>,
    similar_movies: Vec<(usize, f32)>,
    min_budget: u32,
    max_budget: u32,
    search_query: String,
    filtered_indices: Vec<usize>,
    pending_selection: Option<usize>,
    theme: ColorTheme,
}

impl MovieSimilarityApp {
    pub fn load_movies(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(file);

        self.movies = csv_reader
            .deserialize()
            .collect::<Result<Vec<Movie>, _>>()?;

        self.min_budget = self
            .movies
            .iter()
            .map(|movie| movie.budget)
            .min()
            .unwrap_or(0);
        self.max_budget = self
            .movies
            .iter()
            .map(|movie| movie.budget)
            .max()
            .unwrap_or(0);

        self.filtered_indices = (0..self.movies.len()).collect();
        self.theme = ColorTheme::default();

        Ok(())
    }

    fn calculate_similarities(&mut self) {
        if let Some(selected_idx) = self.selected_movie_index {
            let reference_movie = &self.movies[selected_idx];
            let min_budget = self.min_budget;
            let max_budget = self.max_budget;

            let similar_movies_vec: Vec<(usize, f32)> = self
                .movies
                .iter()
                .enumerate()
                .map(|(idx, movie)| {
                    let similarity = movie.similarity(reference_movie, min_budget, max_budget);
                    (idx, similarity)
                })
                .collect();

            self.similar_movies = similar_movies_vec;

            self.similar_movies
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    fn filter_movies(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_indices = self
            .movies
            .iter()
            .enumerate()
            .filter(|(_, movie)| movie.title.to_lowercase().contains(&query))
            .map(|(idx, _)| idx)
            .collect();
    }

    fn process_pending_selection(&mut self) {
        if let Some(idx) = self.pending_selection.take() {
            self.selected_movie_index = Some(idx);
            self.calculate_similarities();
            self.search_query = self.movies[idx].title.clone();
            self.filter_movies();
        }
    }

    fn draw_card(&self, ui: &mut egui::Ui, movie: &Movie, selected: bool) -> egui::Response {
        let frame = egui::Frame::new()
            .fill(if selected {
                self.theme.selected_bg
            } else {
                self.theme.card_bg
            })
            .stroke(Stroke::new(
                1.0,
                if selected {
                    self.theme.primary
                } else {
                    self.theme.border_light
                },
            ))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(10))
            .outer_margin(Margin::same(4));

        let response = frame
            .show(ui, |ui| {
                ui.add(egui::Label::new(
                    RichText::new(&movie.title)
                        .strong()
                        .size(16.0)
                        .color(if selected {
                            self.theme.primary_dark
                        } else {
                            self.theme.text_primary
                        }),
                ))
            })
            .response;

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }
        response.interact(egui::Sense::click())
    }
}

impl eframe::App for MovieSimilarityApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending_selection();

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(8.0, 8.0);
        style.visuals.widgets.noninteractive.bg_fill = self.theme.background;
        style.visuals.widgets.inactive.bg_fill = self.theme.card_bg;
        style.visuals.widgets.hovered.bg_fill = self.theme.primary_light;
        style.visuals.widgets.active.bg_fill = self.theme.primary;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        style.visuals.window_corner_radius = CornerRadius::same(10);
        style.visuals.window_shadow.blur = 5;
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("Movie Similarity Finder")
                        .size(28.0)
                        .color(self.theme.primary)
                        .strong(),
                ));
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(10.0);
            });

            if !self.movies.is_empty() {
                egui::Frame::new()
                    .fill(self.theme.card_bg)
                    .stroke(Stroke::new(1.0, self.theme.border_light))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(1))
                    .outer_margin(Margin::same(5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(
                                RichText::new("🔍 Search:")
                                    .size(16.0)
                                    .strong()
                                    .color(self.theme.text_primary),
                            ));
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut self.search_query)
                                        .desired_width(ui.available_width()),
                                )
                                .changed()
                            {
                                self.filter_movies();
                            }
                        });
                    });

                ui.add_space(10.0);

                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("Select a movie:")
                                .size(18.0)
                                .color(self.theme.primary)
                                .strong(),
                        ));
                        ui.add_space(5.0);

                        egui::ScrollArea::vertical()
                            .id_salt("movie_list")
                            .show(ui, |ui| {
                                for &idx in &self.filtered_indices {
                                    let movie = &self.movies[idx];
                                    let selected = Some(idx) == self.selected_movie_index;

                                    if self.draw_card(ui, movie, selected).clicked() {
                                        self.pending_selection = Some(idx);
                                    }
                                }
                            });
                    });

                    columns[1].vertical(|ui| {
                        if let Some(selected_idx) = self.selected_movie_index {
                            let selected_movie = &self.movies[selected_idx];

                            egui::Frame::new()
                                .fill(self.theme.card_bg)
                                .stroke(Stroke::new(1.0, self.theme.primary))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .outer_margin(Margin::same(5))
                                .show(ui, |ui| {
                                    ui.add(egui::Label::new(
                                        RichText::new("Selected Movie")
                                            .size(18.0)
                                            .color(self.theme.primary)
                                            .strong(),
                                    ));
                                    ui.add_space(5.0);
                                    ui.separator();
                                    ui.add_space(5.0);

                                    ui.add(egui::Label::new(
                                        RichText::new(&selected_movie.title)
                                            .size(20.0)
                                            .strong()
                                            .color(self.theme.text_primary),
                                    ));
                                    ui.add_space(5.0);

                                    ui.horizontal(|ui| {
                                        ui.add(egui::Label::new(
                                            RichText::new(format!(
                                                "Year: {}",
                                                selected_movie.release_date
                                            ))
                                            .size(14.0)
                                            .color(self.theme.text_secondary),
                                        ));
                                        ui.add(egui::Label::new(
                                            RichText::new(format!(
                                                "Rating: {:.1}",
                                                selected_movie.vote_average
                                            ))
                                            .size(14.0)
                                            .color(self.theme.text_secondary),
                                        ));
                                    });

                                    ui.add(egui::Label::new(
                                        RichText::new(format!(
                                            "Budget: ${}",
                                            selected_movie.budget
                                        ))
                                        .size(14.0)
                                        .color(self.theme.text_secondary),
                                    ));

                                    ui.collapsing(
                                        RichText::new("More Details")
                                            .size(14.0)
                                            .color(self.theme.primary),
                                        |ui| {
                                            if !selected_movie.genres.is_empty() {
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.add(egui::Label::new(
                                                        RichText::new("Genres:")
                                                            .strong()
                                                            .color(self.theme.text_primary),
                                                    ));
                                                    for genre in &selected_movie.genres {
                                                        ui.label(
                                                            RichText::new(genre.to_string())
                                                                .color(self.theme.text_secondary),
                                                        );
                                                    }
                                                });
                                            }

                                            if !selected_movie.homepage.is_empty() {
                                                ui.horizontal(|ui| {
                                                    ui.add(egui::Label::new(
                                                        RichText::new("Homepage:")
                                                            .strong()
                                                            .color(self.theme.text_primary),
                                                    ));
                                                    ui.label(
                                                        RichText::new(&selected_movie.homepage)
                                                            .color(self.theme.text_secondary),
                                                    );
                                                });
                                            }

                                            if !selected_movie.keywords.is_empty() {
                                                ui.add(egui::Label::new(
                                                    RichText::new("Keywords:")
                                                        .strong()
                                                        .color(self.theme.text_primary),
                                                ));
                                                ui.horizontal_wrapped(|ui| {
                                                    for keyword in &selected_movie.keywords {
                                                        ui.label(
                                                            RichText::new(keyword.to_string())
                                                                .color(self.theme.text_secondary),
                                                        );
                                                    }
                                                });
                                            }

                                            if !selected_movie.production_companies.is_empty() {
                                                ui.add(egui::Label::new(
                                                    RichText::new("Production Companies:")
                                                        .strong()
                                                        .color(self.theme.text_primary),
                                                ));
                                                for company in &selected_movie.production_companies
                                                {
                                                    ui.label(
                                                        RichText::new(company.to_string())
                                                            .color(self.theme.text_secondary),
                                                    );
                                                }
                                            }
                                        },
                                    );
                                });

                            ui.add_space(10.0);

                            ui.add(egui::Label::new(
                                RichText::new(format!("Top {} Similar Movies:", TOP_N))
                                    .size(18.0)
                                    .color(self.theme.primary)
                                    .strong(),
                            ));
                            ui.add_space(5.0);

                            let mut count = 0;
                            let mut index = 0;
                            let similar_indices: Vec<(usize, f32)> = self.similar_movies.clone();

                            egui::ScrollArea::vertical()
                                .id_salt("similar_movies")
                                .show(ui, |ui| {
                                    while count < TOP_N && index < similar_indices.len() {
                                        let (movie_idx, similarity) = similar_indices[index];
                                        index += 1;

                                        if movie_idx == selected_idx {
                                            continue;
                                        }

                                        let similar_movie = &self.movies[movie_idx];

                                        let response = egui::Frame::new()
                                            .fill(self.theme.card_bg)
                                            .stroke(Stroke::new(1.0, self.theme.border_light))
                                            .corner_radius(CornerRadius::same(6))
                                            .inner_margin(Margin::same(8))
                                            .outer_margin(Margin::same(4))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.add(egui::Label::new(
                                                        RichText::new(format!("{}.", count + 1))
                                                            .strong()
                                                            .color(self.theme.text_secondary),
                                                    ));

                                                    ui.add(egui::Label::new(
                                                        RichText::new(&similar_movie.title)
                                                            .color(self.theme.primary),
                                                    ));

                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            let similarity_percentage =
                                                                (similarity * 100.0) as i32;
                                                            ui.add(egui::Label::new(
                                                                RichText::new(format!(
                                                                    "{}%",
                                                                    similarity_percentage
                                                                ))
                                                                .color(self.theme.secondary)
                                                                .strong(),
                                                            ));
                                                        },
                                                    );
                                                });
                                            })
                                            .response
                                            .interact(egui::Sense::click());

                                        if response.hovered() {
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }

                                        if response.clicked() {
                                            self.pending_selection = Some(movie_idx);
                                        }
                                        count += 1;
                                    }
                                });
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.add(egui::Label::new(
                                    RichText::new("Select a movie from the list")
                                        .size(18.0)
                                        .color(self.theme.text_secondary),
                                ));
                                ui.add_space(10.0);
                                ui.add(egui::Label::new(
                                    RichText::new("to see details and similar titles")
                                        .size(16.0)
                                        .color(self.theme.text_secondary),
                                ));
                            });
                        }
                    });
                });
            }
        });
    }
}
