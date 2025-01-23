use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ACCEPT};
use std::error::Error;

// const BASE_CRAWLER_API: &str = "https://google-crawler-seven.vercel.app/api/%s/search";
// const BASE_CRAWLER_API: &str = "https://google-crawler.onrender.com/api/%s/search";
const BASE_CRAWLER_API: &str = "https://wondrous-basbousa-334609.netlify.app/api/%s/search";
// const BASE_CRAWLER_API: &str = "https://google-crawler-blfu.vercel.app/api/%s/search";
// const BASE_CRAWLER_API: &str = "http://localhost:3000/api/%s/search";

const API_SECRET: &str = "LNSHjP2F8K+RCT3j60JmFJyGJzsBSHbOzJ1bGpKQb1w=";

#[derive(Debug, Serialize)]
struct CrawlerRequest {
    phrase: String,
    start: i32,
    num_to_crawl: i32,
    existing_urls_length: i32,
    language: LanguageConfig,
    blacklist: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CrawlerResponse {
    pub success: bool,
    pub data: Vec<SearchResult>,
    pub engine: String,
    pub cost: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}

pub fn get_blacklist() -> Vec<String> {
    vec![
        "pagesjaunes.fr", "fnac.com", "travaux.com", "pagespro.com",
        "facebook.com", "youtube.com", "netflix.com", "instagram.com",
        "twitter.com", "linkedin.com", "x.com", "tiktok.com",
        "boulanger.com", "amazon.com", "alibaba.com", "walmart.com",
        "cdiscount.com", "myntra.com", "dailymotion.com",
    ].into_iter().map(String::from).collect()
}

#[derive(Debug, Serialize, Clone)]
pub struct LanguageConfig {
    pub query: String,
    pub header: String,
}

pub fn setup_language(language: &str, engine: &str) -> Option<LanguageConfig> {
    let config = match engine {
        "bing" => match language {
            "fr" => ("setlang=fr&cc=FR", "fr-FR,fr;q=0.9"),
            "en" => ("setlang=en&cc=US", "en-US,en;q=0.9"),
            _ => return None,
        },
        "yahoo" => match language {
            "fr" => ("vl=lang_fr&fl=fr", "fr-FR,fr;q=0.9"),
            "en" => ("vl=lang_en&fl=en", "en-US,en;q=0.9"),
            _ => return None,
        },
        "duckduckgo" => match language {
            "fr" => ("fr-fr", "fr-FR,fr;q=0.9"),
            "en" => ("us-en", "en-US,en;q=0.9"),
            _ => return None,
        },
        "google" => match language {
            "fr" => ("hl=fr&lr=lang_fr", "fr-FR,fr;q=0.9"),
            "en" => ("hl=en&lr=lang_en", "en-US,en;q=0.9"),
            _ => return None,
        },
        _ => return None,
    };

    Some(LanguageConfig {
        query: config.0.to_string(),
        header: config.1.to_string(),
    })
}

pub async fn call_search_engine_crawler(
    engine: &str,
    phrase: &str,
    start: i32,
    num_to_crawl: i32,
    existing_urls_length: i32,
    language: &str,
    blacklist: Vec<String>,
) -> Result<Option<CrawlerResponse>, Box<dyn Error>> {
    let api_url = format!("{}", BASE_CRAWLER_API.replace("%s", engine));
    println!("Appel API vers: {}", api_url);

    let lang_config = setup_language(language, engine)
        .ok_or("Configuration de langue non trouvée")?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert("x-auth-secret", HeaderValue::from_str(API_SECRET)?);
    headers.insert("Accept-Language", HeaderValue::from_str(&lang_config.header)?);

    let request_body = CrawlerRequest {
        phrase: phrase.to_string(),
        start,
        num_to_crawl,
        existing_urls_length,
        language: lang_config,
        blacklist,
    };

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let response = client
        .post(&api_url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;
    println!("Code HTTP: {}", status);
    println!("Réponse: {}", response_text);

    if !status.is_success() {
        println!("Erreur HTTP {} pour {}. Réponse: {}", status, engine, response_text);
        return Ok(None);
    }

    match serde_json::from_str(&response_text) {
        Ok(result) => Ok(Some(result)),
        Err(e) => {
            println!("Erreur de parsing JSON pour {}: {}", engine, e);
            Ok(None)
        }
    }
} 