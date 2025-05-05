use crate::movie::Movie;
use eframe::egui;
use std::{fs::File, path::Path};

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
}

impl eframe::App for MovieSimilarityApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending_selection();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Movie Similarity Finder");

            if !self.movies.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    if ui.text_edit_singleline(&mut self.search_query).changed() {
                        self.filter_movies();
                    }
                });

                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Select a movie:");

                    for &idx in &self.filtered_indices {
                        let movie = &self.movies[idx];
                        let selected = Some(idx) == self.selected_movie_index;

                        if ui.selectable_label(selected, &movie.title).clicked() {
                            self.pending_selection = Some(idx);
                        }
                    }
                });

                ui.add_space(20.0);

                if let Some(selected_idx) = self.selected_movie_index {
                    let selected_movie = &self.movies[selected_idx];

                    ui.heading("Selected Movie:");
                    ui.label(format!("Title: {}", selected_movie.title));
                    ui.label(format!("Year: {}", selected_movie.release_date));
                    ui.label(format!("Budget: ${}", selected_movie.budget));
                    ui.label(format!("Rating: {:.1}", selected_movie.vote_average));

                    ui.add_space(10.0);
                    ui.heading(format!("Top {} Similar Movies:", TOP_N));

                    let mut count = 0;
                    let mut index = 0;

                    let similar_indices: Vec<(usize, f32)> = self.similar_movies.clone();

                    while count < TOP_N && index < similar_indices.len() {
                        let (movie_idx, similarity) = similar_indices[index];
                        index += 1;

                        if movie_idx == selected_idx {
                            continue;
                        }

                        let similar_movie = &self.movies[movie_idx];

                        ui.horizontal(|ui| {
                            ui.label(format!("{}. ", count + 1));
                            if ui.selectable_label(false, &similar_movie.title).clicked() {
                                self.pending_selection = Some(movie_idx);
                            }
                            ui.label(format!("(Similarity: {:.4})", similarity));
                        });

                        count += 1;
                    }
                }
            }
        });
    }
}
