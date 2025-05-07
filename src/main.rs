use movie_cbr::gui::MovieSimilarityApp;
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    // Initialize with default options
    let native_options = eframe::NativeOptions {
        ..Default::default()
    };

    // Create app and load movies
    let mut app = MovieSimilarityApp::default();
    let path = "./data/tmdb_5000_movies.csv";
    if let Err(err) = app.load_movies(Path::new(path)) {
        eprintln!("Error loading movies: {}", err);
    }

    // Run the application
    eframe::run_native(
        "Movie Similarity Finder",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
