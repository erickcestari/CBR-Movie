use cbr::movie::Movie;
use std::{env, process::exit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cbr <movies.csv>");
        exit(64)
    }

    let path = &args[1];
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)?;

    let movies: Vec<Movie> = csv_reader
        .deserialize()
        .collect::<Result<Vec<Movie>, _>>()?;

    let min_budget = movies.iter().map(|movie| movie.budget).min().unwrap_or(0);
    let max_budget = movies.iter().map(|movie| movie.budget).max().unwrap_or(0);

    let reference_movie = &movies[0];
    println!("Reference movie: {}", reference_movie.title);

    let mut movie_similarities: Vec<(&Movie, f32)> = movies.iter()
        .map(|movie| {
            let similarity = movie.similarity(reference_movie, min_budget, max_budget);
            (movie, similarity)
        })
        .collect();

    movie_similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nTop 5 similar movies:");
    let mut count = 0;
    let mut i = 0;
    
    while count < 5 && i < movie_similarities.len() {
        let (movie, similarity) = movie_similarities[i];
        
        if movie.id != reference_movie.id {
            println!("{}. {} (Similarity: {:.4})", count + 1, movie.title, similarity);
            count += 1;
        }
        
        i += 1;
    }

    Ok(())
}