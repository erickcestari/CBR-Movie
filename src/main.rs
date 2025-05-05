use std::path::Path;
use movie_cbr::gui::MovieSimilarityApp;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        ..Default::default()
    };

    let mut app = MovieSimilarityApp::default();
    let path = "./data/tmdb_5000_movies.csv";
    if let Err(err) = app.load_movies(Path::new(path)) {
        eprintln!("Error loading movies: {}", err);
    }

    eframe::run_native(
        "Movie Similarity Finder",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}