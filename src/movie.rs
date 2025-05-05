use serde::{Deserialize, Serialize};

use crate::cbr::{self, HasId};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Movie {
    pub budget: u32,
    #[serde(deserialize_with = "deserialize_json_string")]
    genres: Vec<Genre>,
    homepage: String,
    pub id: u32,
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
    pub title: String,
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

impl HasId for Keyword {
    fn id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Company {
    id: u32,
    name: String,
}

impl HasId for Company {
    fn id(&self) -> u32 {
        self.id
    }
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

const BUDGET_WEIGHT: f32 = 0.5;
    const GENRES_WEIGHT: f32 = 2.0;
    const HOMEPAGE_WEIGHT: f32 = 0.2;
    const KEYWORDS_WEIGHT: f32 = 1.5;
    const OVERVIEW_WEIGHT: f32 = 1.0;
    const PRODUCTION_COMPANIES_WEIGHT: f32 = 0.8;
    const TITLE_WEIGHT: f32 = 0.7;
    const TOTAL_WEIGHT: f32 = BUDGET_WEIGHT + GENRES_WEIGHT + HOMEPAGE_WEIGHT + KEYWORDS_WEIGHT + OVERVIEW_WEIGHT + PRODUCTION_COMPANIES_WEIGHT + TITLE_WEIGHT;

impl Movie {
    pub fn similarity(&self, other: &Movie, min_budget: u32, max_budget: u32) -> f32 {
        let budget_diff = cbr::similarity_number(self.budget, other.budget, max_budget, min_budget) * BUDGET_WEIGHT;
        let genres_diff = cbr::similarity_id(&self.genres, &other.genres) * GENRES_WEIGHT;
        let homepage_diff = cbr::similarity_string(&self.homepage, &other.homepage) * HOMEPAGE_WEIGHT;
        let keywords_diff = cbr::similarity_id(&self.keywords, &other.keywords) * KEYWORDS_WEIGHT;
        let overview_diff = cbr::similarity_string(&self.overview, &other.overview) * OVERVIEW_WEIGHT;
        let production_companies_diff = cbr::similarity_id(&self.production_companies, &other.production_companies) * PRODUCTION_COMPANIES_WEIGHT;
        let title_diff = cbr::similarity_string(&self.title, &other.title) * TITLE_WEIGHT;

        let result = budget_diff + genres_diff + homepage_diff + keywords_diff + overview_diff + production_companies_diff + title_diff;

        result / TOTAL_WEIGHT
    }
}
