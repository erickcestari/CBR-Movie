use crate::cbr::{self, HasId};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Represents a movie with all its attributes
/// Used for case-based reasoning to find similar movies
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Movie {
    /// Budget of the movie in some currency unit (likely USD)
    pub budget: u32,
    /// List of genres associated with the movie
    /// Note: Deserialized from JSON string in CSV
    #[serde(deserialize_with = "deserialize_json_string")]
    pub genres: Vec<Genre>,
    /// Official website URL of the movie
    pub homepage: String,
    /// Unique identifier for the movie
    pub id: u32,
    /// List of keywords associated with the movie
    /// Note: Deserialized from JSON string in CSV
    #[serde(deserialize_with = "deserialize_json_string")]
    pub keywords: Vec<Keyword>,
    /// Original language of the movie
    original_language: String,
    /// Original title of the movie in its native language
    original_title: String,
    /// Brief summary of the movie's plot
    overview: String,
    /// Popularity score of the movie (algorithm-dependent)
    popularity: f32,
    /// Production companies involved in the movie
    /// Note: Deserialized from JSON string in CSV
    #[serde(deserialize_with = "deserialize_json_string")]
    pub production_companies: Vec<Company>,
    /// Countries where the movie was produced
    /// Note: Deserialized from JSON string in CSV
    #[serde(deserialize_with = "deserialize_json_string")]
    production_countries: Vec<Country>,
    /// Release date of the movie (format: YYYY-MM-DD)
    pub release_date: String,
    /// Box office revenue of the movie
    revenue: String,
    /// Duration of the movie in minutes (may be null)
    runtime: Option<f32>,
    /// Languages spoken in the movie
    /// Note: Deserialized from JSON string in CSV
    #[serde(deserialize_with = "deserialize_json_string")]
    spoken_languages: Vec<Language>,
    /// Production status (e.g., "Released", "In Production")
    status: String,
    /// Marketing tagline of the movie
    tagline: String,
    /// Display title of the movie
    pub title: String,
    /// Average user rating (likely on a scale of 0-10)
    pub vote_average: f32,
    /// Number of user votes/ratings
    pub vote_count: u32,
}

/// Implementation of HasId trait for Movie
/// Allows Movie objects to be used in similarity calculations
impl HasId for Movie {
    /// Returns the unique identifier for this movie
    fn id(&self) -> u32 {
        self.id
    }
}

/// Represents a movie genre
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Genre {
    /// Unique identifier for the genre
    id: u32,
    /// Human-readable name of the genre (e.g., "Action", "Comedy")
    name: String,
}

/// Implements Display trait for Genre to allow printing
impl Display for Genre {
    /// Returns the name of the genre when displayed
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Implementation of HasId trait for Genre
/// Allows Genre objects to be used in similarity calculations
impl HasId for Genre {
    /// Returns the unique identifier for this genre
    fn id(&self) -> u32 {
        self.id
    }
}

/// Represents a keyword associated with a movie
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Keyword {
    /// Unique identifier for the keyword
    id: u32,
    /// Human-readable name of the keyword (e.g., "dystopia", "space")
    name: String,
}

/// Implementation of HasId trait for Keyword
/// Allows Keyword objects to be used in similarity calculations
impl HasId for Keyword {
    /// Returns the unique identifier for this keyword
    fn id(&self) -> u32 {
        self.id
    }
}

/// Implements Display trait for Keyword to allow printing
impl Display for Keyword {
    /// Returns the name of the keyword when displayed
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Represents a production company
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Company {
    /// Unique identifier for the company
    id: u32,
    /// Name of the production company
    name: String,
}

/// Implementation of HasId trait for Company
/// Allows Company objects to be used in similarity calculations
impl HasId for Company {
    /// Returns the unique identifier for this company
    fn id(&self) -> u32 {
        self.id
    }
}

/// Implements Display trait for Company to allow printing
impl Display for Company {
    /// Returns the name of the company when displayed
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Represents a country
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Country {
    /// ISO 3166-1 country code (e.g., "US", "FR")
    iso_3166_1: String,
    /// Full name of the country
    name: String,
}

/// Represents a language
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Language {
    /// ISO 639-1 language code (e.g., "en", "fr")
    iso_639_1: String,
    /// Full name of the language
    name: String,
}

/// Custom deserializer function for parsing JSON strings embedded in CSV cells
///
/// This function takes a string that contains serialized JSON data and converts it
/// to the appropriate Rust type (e.g., Vec<Genre>, Vec<Keyword>)
///
/// # Arguments
/// * `deserializer` - The deserializer provided by serde
///
/// # Returns
/// * `Result<T, D::Error>` - The deserialized value or an error
fn deserialize_json_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: for<'a> serde::Deserialize<'a>,
{
    // First deserialize the input as a string
    let s = String::deserialize(deserializer)?;
    // Then parse that string as JSON to get the desired type
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

/// Weight constants for similarity calculation
/// Higher values give more importance to that attribute when calculating similarity
const BUDGET_WEIGHT: f32 = 0.3;
const GENRES_WEIGHT: f32 = 1.0;
const HOMEPAGE_WEIGHT: f32 = 0.2;
const KEYWORDS_WEIGHT: f32 = 2.0;
const PRODUCTION_COMPANIES_WEIGHT: f32 = 1.0;
const TITLE_WEIGHT: f32 = 2.5;
/// Sum of all weights used for normalization
const TOTAL_WEIGHT: f32 = BUDGET_WEIGHT
    + GENRES_WEIGHT
    + HOMEPAGE_WEIGHT
    + KEYWORDS_WEIGHT
    + PRODUCTION_COMPANIES_WEIGHT
    + TITLE_WEIGHT;

impl Movie {
    /// Calculates the similarity between this movie and another movie
    ///
    /// The similarity is based on multiple attributes with different weights.
    /// Each attribute contributes to the overall similarity score based on its importance.
    /// The final similarity score is normalized to be between 0.0 and 1.0.
    ///
    /// # Arguments
    /// * `other` - The movie to compare with
    /// * `min_budget` - The minimum budget in the dataset (for normalization)
    /// * `max_budget` - The maximum budget in the dataset (for normalization)
    ///
    /// # Returns
    /// * `f32` - A similarity score between 0.0 (completely different) and 1.0 (identical)
    ///
    pub fn similarity(&self, other: &Movie, min_budget: u32, max_budget: u32) -> f32 {
        // Calculate budget similarity (normalized by min/max values)
        let budget_diff = cbr::similarity_number(self.budget, other.budget, max_budget, min_budget)
            * BUDGET_WEIGHT;
        // Calculate genre similarity (based on common genres)
        let genres_diff = cbr::similarity_id(&self.genres, &other.genres) * GENRES_WEIGHT;
        // Calculate homepage similarity (string comparison)
        let homepage_diff =
            cbr::similarity_string(&self.homepage, &other.homepage) * HOMEPAGE_WEIGHT;
        // Calculate keyword similarity (based on common keywords)
        let keywords_diff = cbr::similarity_id(&self.keywords, &other.keywords) * KEYWORDS_WEIGHT;
        // Calculate production company similarity
        let production_companies_diff =
            cbr::similarity_id(&self.production_companies, &other.production_companies)
                * PRODUCTION_COMPANIES_WEIGHT;
        // Calculate title similarity (string comparison)
        let title_diff = cbr::similarity_string(&self.title, &other.title) * TITLE_WEIGHT;

        // Sum all the weighted similarities
        let result = budget_diff
            + genres_diff
            + homepage_diff
            + keywords_diff
            + production_companies_diff
            + title_diff;

        // Normalize by total weight to get a value between 0.0 and 1.0
        result / TOTAL_WEIGHT
    }
}
