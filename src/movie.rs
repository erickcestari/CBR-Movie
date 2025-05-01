use serde::{Deserialize, Serialize};

use crate::cbr::{self, HasId};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Movie {
    budget: u32,
    #[serde(deserialize_with = "deserialize_json_string")]
    genres: Vec<Genre>,
    homepage: String,
    id: u32,
    #[serde(deserialize_with = "deserialize_json_string")]
    keywords: Vec<Keyword>,
    original_language: String,
    original_title: String,
    overview: String,
    popularity: f32,
    #[serde(deserialize_with = "deserialize_json_string")]
    production_companies: Vec<Company>,
    #[serde(deserialize_with = "deserialize_json_string")]
    production_countries: Vec<Country>,
    release_date: String,
    revenue: String,
    runtime: Option<f32>,
    #[serde(deserialize_with = "deserialize_json_string")]
    spoken_languages: Vec<Language>,
    status: String,
    tagline: String,
    title: String,
    vote_average: f32,
    vote_count: u32,
}

impl HasId for Movie {
    fn id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Genre {
    id: u32,
    name: String,
}

impl HasId for Genre {
  fn id(&self) -> u32 {
      self.id
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Keyword {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Company {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Country {
    iso_3166_1: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Language {
    iso_639_1: String,
    name: String,
}

// Custom deserializer for JSON strings in CSV cells
fn deserialize_json_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: for<'a> serde::Deserialize<'a>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

const MAX_BUDGET: u32 = 1000000000;
const MIN_BUDGET: u32 = 0;

impl Movie {
    pub fn similarity(&self, other: &Movie) -> f32 {
        let budget_diff = cbr::similarity_number(self.budget, other.budget, MAX_BUDGET, MIN_BUDGET);
        let genres_diff = cbr::similarity_id(&self.genres, &other.genres);
        let homepage_diff = cbr::similarity_string(&self.homepage, &other.homepage);

        budget_diff + genres_diff + homepage_diff
    }
}
