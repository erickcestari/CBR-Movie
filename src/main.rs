use std::{env, process::exit};
use cbr::movie::Movie;

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

    let first_movie = &movies.clone()[0];

    for movie in movies {
        movie.similarity(first_movie);
    }

    Ok(())
}
