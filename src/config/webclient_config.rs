use std::fmt::Display;

use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct WebClientConfig {
    pub search_url: String,
    pub provider: AvailableSearchEngines,
}

impl WebClientConfig {
    pub fn new<U: IntoUrl>(u: U) -> Self {
        if !u.into_url().is_ok(){

        }
        todo!()
    }
}

#[derive(Debug, Copy, Default, Clone, Serialize, Deserialize)]
pub enum AvailableSearchEngines {
    #[default]
    SearXNG,
}

impl Display for AvailableSearchEngines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            AvailableSearchEngines::SearXNG => {
                write!(f, "searxng")
            }
        }
    }
}

impl Default for WebClientConfig {
    fn default() -> Self {
        Self {
            search_url: String::from("http://localhost:8080"),
            provider: Default::default(),
        }
    }
}
