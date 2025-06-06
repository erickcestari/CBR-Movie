// Import necessary modules and types from the crate and external dependencies
use crate::movie::Movie; // Import the Movie struct from the local movie module
use eframe::egui::{self, CursorIcon, Margin}; // Import egui and related components for GUI
use egui::{Color32, CornerRadius, RichText, Stroke, Vec2}; // Import specific egui types for styling
use std::{fs::File, path::Path}; // Import File and Path from standard library for file operations

/// ColorTheme defines the color palette used throughout the application
/// This struct centralizes all color definitions for consistent UI styling
struct ColorTheme {
    primary: Color32,        // Main brand/accent color
    primary_light: Color32,  // Lighter version of primary color
    primary_dark: Color32,   // Darker version of primary color
    secondary: Color32,      // Secondary color for contrasting elements
    background: Color32,     // Main application background color
    card_bg: Color32,        // Background color for card elements
    text_primary: Color32,   // Main text color
    text_secondary: Color32, // Secondary text color (for less emphasis)
    border_light: Color32,   // Light border color
    selected_bg: Color32,    // Background color for selected elements
}

/// Default implementation for ColorTheme
/// Sets up a dark theme with orange/brown accent colors
impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme {
            primary: Color32::from_rgb(210, 144, 84), // Orange-brown
            primary_light: Color32::from_rgb(237, 184, 121), // Light orange-brown
            primary_dark: Color32::from_rgb(160, 95, 50), // Dark orange-brown
            secondary: Color32::from_rgb(235, 235, 235), // Light gray
            background: Color32::from_rgb(24, 24, 24), // Very dark gray (almost black)
            card_bg: Color32::from_rgb(36, 36, 36),   // Dark gray (for cards)
            text_primary: Color32::from_rgb(235, 235, 235), // Light gray text
            text_secondary: Color32::from_rgb(160, 160, 160), // Medium gray text
            border_light: Color32::from_rgb(64, 64, 64), // Medium-dark gray border
            selected_bg: Color32::from_rgb(54, 45, 38), // Dark brown-gray for selection
        }
    }
}

// Constant defining how many similar movies to display
const TOP_N: usize = 10;

/// Main application struct for the Movie Similarity App
/// Contains all state needed to run the application
#[derive(Default)]
pub struct MovieSimilarityApp {
    movies: Vec<Movie>,                  // List of all movies loaded from CSV
    selected_movie_index: Option<usize>, // Currently selected movie (if any)
    similar_movies: Vec<(usize, f32)>,   // List of similar movies with similarity scores
    min_budget: u32,                     // Minimum movie budget (used for normalization)
    max_budget: u32,                     // Maximum movie budget (used for normalization)
    search_query: String,                // Current search query text
    filtered_indices: Vec<usize>,        // Indices of movies matching the search query
    pending_selection: Option<usize>,    // Movie selection that hasn't been processed yet
    theme: ColorTheme,                   // Color theme for the application
}

impl MovieSimilarityApp {
    /// Loads movie data from a CSV file at the specified path
    ///
    /// # Arguments
    /// * `path` - Path to the CSV file containing movie data
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error
    ///
    /// This function reads the CSV, deserializes it to Movie objects,
    /// initializes the min/max budget values for later normalization,
    /// and sets up the filtered indices list.
    pub fn load_movies(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Open the file at the specified path
        let file = File::open(path)?;

        // Create a CSV reader with headers and flexible parsing
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(file);

        // Deserialize the CSV rows into Movie objects
        self.movies = csv_reader
            .deserialize()
            .collect::<Result<Vec<Movie>, _>>()?;

        // Find minimum budget across all movies (for normalization)
        self.min_budget = self
            .movies
            .iter()
            .map(|movie| movie.budget)
            .min()
            .unwrap_or(0);

        // Find maximum budget across all movies (for normalization)
        self.max_budget = self
            .movies
            .iter()
            .map(|movie| movie.budget)
            .max()
            .unwrap_or(0);

        // Initialize filtered_indices with all movie indices
        self.filtered_indices = (0..self.movies.len()).collect();

        // Initialize the color theme
        self.theme = ColorTheme::default();

        Ok(())
    }

    /// Calculates similarity scores between the selected movie and all other movies
    ///
    /// This function is called when a movie is selected. It:
    /// 1. Gets the selected movie as the reference
    /// 2. Calculates similarity for each movie against the reference
    /// 3. Sorts the results by similarity score in descending order
    fn calculate_similarities(&mut self) {
        if let Some(selected_idx) = self.selected_movie_index {
            let reference_movie = &self.movies[selected_idx];
            let min_budget = self.min_budget;
            let max_budget = self.max_budget;

            // Calculate similarity scores for all movies compared to the reference
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

            // Sort movies by similarity score (descending)
            self.similar_movies
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    /// Filters the movies based on the current search query
    ///
    /// Updates filtered_indices to contain only indices of movies
    /// whose titles contain the search query (case insensitive)
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

    /// Processes any pending movie selection
    ///
    /// When a movie is clicked, the selection is stored as "pending"
    /// and processed here to avoid borrow checker issues. This function:
    /// 1. Takes the pending selection (if any)
    /// 2. Updates the selected movie index
    /// 3. Recalculates similarities
    /// 4. Updates the search query to the selected movie's title
    /// 5. Filters the movie list accordingly
    fn process_pending_selection(&mut self) {
        if let Some(idx) = self.pending_selection.take() {
            self.selected_movie_index = Some(idx);
            self.calculate_similarities();
            self.search_query = self.movies[idx].title.clone();
            self.filter_movies();
        }
    }

    /// Draws a movie card UI element with appropriate styling
    ///
    /// # Arguments
    /// * `ui` - The egui UI to draw on
    /// * `movie` - The movie to display in the card
    /// * `selected` - Whether this movie is currently selected
    ///
    /// # Returns
    /// * `egui::Response` - The UI response for interaction handling
    ///
    /// Creates a styled card for a movie with appropriate colors and
    /// interactions (hover cursor, click sensing)
    fn draw_card(&self, ui: &mut egui::Ui, movie: &Movie, selected: bool) -> egui::Response {
        // Create a frame with appropriate styling based on selection state
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

        // Show the frame with the movie title
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

        // Show pointing hand cursor on hover
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // Make the card clickable
        response.interact(egui::Sense::click())
    }
}

/// Implementation of the eframe::App trait for MovieSimilarityApp
/// This handles the main rendering and UI update loop
impl eframe::App for MovieSimilarityApp {
    /// Updates the application state and renders the UI
    ///
    /// # Arguments
    /// * `ctx` - The egui context
    /// * `_frame` - The eframe frame (unused)
    ///
    /// This is called each frame to update the application and render the UI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending movie selection
        self.process_pending_selection();

        // Set up the application style based on the theme
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

        // Create the central panel for the main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            // App title at the top
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

            // Main application content (only shown if movies are loaded)
            if !self.movies.is_empty() {
                // Search bar
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

                // Two-column layout: movies list and details panel
                ui.columns(2, |columns| {
                    // Left column: Movie list
                    columns[0].vertical(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("Select a movie:")
                                .size(18.0)
                                .color(self.theme.primary)
                                .strong(),
                        ));
                        ui.add_space(5.0);

                        // Scrollable list of movie cards
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

                    // Right column: Selected movie details and similar movies
                    columns[1].vertical(|ui| {
                        if let Some(selected_idx) = self.selected_movie_index {
                            let selected_movie = &self.movies[selected_idx];

                            // Selected movie details panel
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

                                    // Movie title
                                    ui.add(egui::Label::new(
                                        RichText::new(&selected_movie.title)
                                            .size(20.0)
                                            .strong()
                                            .color(self.theme.text_primary),
                                    ));
                                    ui.add_space(5.0);

                                    // Year and rating
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

                                    // Budget
                                    ui.add(egui::Label::new(
                                        RichText::new(format!(
                                            "Budget: ${}",
                                            selected_movie.budget
                                        ))
                                        .size(14.0)
                                        .color(self.theme.text_secondary),
                                    ));

                                    // Collapsible "More Details" section
                                    ui.collapsing(
                                        RichText::new("More Details")
                                            .size(14.0)
                                            .color(self.theme.primary),
                                        |ui| {
                                            // Genres
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

                                            // Homepage
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

                                            // Keywords
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

                                            // Production Companies
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

                            // Similar movies section
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

                            // Scrollable list of similar movies
                            egui::ScrollArea::vertical()
                                .id_salt("similar_movies")
                                .show(ui, |ui| {
                                    while count < TOP_N && index < similar_indices.len() {
                                        let (movie_idx, similarity) = similar_indices[index];
                                        index += 1;

                                        // Skip the reference movie itself
                                        if movie_idx == selected_idx {
                                            continue;
                                        }

                                        let similar_movie = &self.movies[movie_idx];

                                        // Create a card for each similar movie
                                        let response = egui::Frame::new()
                                            .fill(self.theme.card_bg)
                                            .stroke(Stroke::new(1.0, self.theme.border_light))
                                            .corner_radius(CornerRadius::same(6))
                                            .inner_margin(Margin::same(8))
                                            .outer_margin(Margin::same(4))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    // Ranking number
                                                    ui.add(egui::Label::new(
                                                        RichText::new(format!("{}.", count + 1))
                                                            .strong()
                                                            .color(self.theme.text_secondary),
                                                    ));

                                                    // Movie title
                                                    ui.add(egui::Label::new(
                                                        RichText::new(&similar_movie.title)
                                                            .color(self.theme.primary),
                                                    ));

                                                    // Similarity percentage (right-aligned)
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

                                        // Show pointing hand cursor on hover
                                        if response.hovered() {
                                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                                        }

                                        // Handle clicks to select this movie
                                        if response.clicked() {
                                            self.pending_selection = Some(movie_idx);
                                        }
                                        count += 1;
                                    }
                                });
                        } else {
                            // Display a message when no movie is selected
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
