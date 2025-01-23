use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::text_analyzer::TextAnalyzer;
use crate::web_analyzer::WebAnalyzer;
use std::error::Error;
use std::collections::HashMap;

// Structures de requête et réponse
#[derive(Deserialize)]
pub struct AnalysisRequest {
    urls: Vec<String>,
    ngrams_to_analyze: Option<Vec<usize>>,
}

#[derive(Serialize)]
pub struct FrequencyResult {
    expression: String,
    gram_type: String,
    #[serde(serialize_with = "serialize_f64_2_decimals")]
    average_occurrences: f64,
    #[serde(serialize_with = "serialize_f64_2_decimals")]
    average_percentage: f64,
    #[serde(serialize_with = "serialize_f64_2_decimals")]
    doc_count_percentage: f64,
    sources: Vec<String>,
}

#[derive(Serialize)]
pub struct DocumentStats {
    url: String,
    total_retained: usize,
    total_unique: usize,
    word_count: usize,
    #[serde(serialize_with = "serialize_f64_2_decimals")]
    average_word_length: f64,
}

#[derive(Serialize)]
pub struct UrlStatus {
    url: String,
    status: String,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct AnalysisResponse {
    frequencies: Vec<FrequencyResult>,
    document_stats: Vec<DocumentStats>,
    url_statuses: Vec<UrlStatus>,
}

// Point d'entrée de l'API
#[post("/api/analyze")]
pub async fn analyze_urls(data: web::Json<AnalysisRequest>) -> impl Responder {
    let urls = data.urls.clone();
    let ngrams = data.ngrams_to_analyze.clone().unwrap_or(vec![1, 2, 3]);
    
    match analyze_content(urls, ngrams).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// Fonctions d'analyse
async fn analyze_content(urls: Vec<String>, ngrams: Vec<usize>) -> Result<AnalysisResponse, Box<dyn Error>> {
    let mut frequencies = HashMap::new();
    let mut doc_stats = Vec::new();
    let mut url_statuses = Vec::new();
    let mut successful_urls = 0;
    
    for url in &urls {
        match analyze_single_url(url, &ngrams, &mut frequencies, &mut doc_stats).await {
            Ok(_) => {
                successful_urls += 1;
                url_statuses.push(create_url_status(url, true, None));
            }
            Err(e) => url_statuses.push(create_url_status(url, false, Some(e.to_string()))),
        }
    }

    if successful_urls == 0 {
        return Err("Aucune URL n'a pu être analysée".into());
    }

    let results = process_frequencies(frequencies, successful_urls);

    Ok(AnalysisResponse {
        frequencies: results,
        document_stats: doc_stats,
        url_statuses,
    })
}

async fn analyze_single_url(
    url: &str,
    ngrams: &[usize],
    frequencies: &mut HashMap<(String, String), (f64, f64, Vec<String>)>,
    doc_stats: &mut Vec<DocumentStats>,
) -> Result<(), Box<dyn Error>> {
    let content = fetch_and_prepare_content(url).await?;
    let mut analyzer = create_analyzer(&content)?;
    
    process_ngrams(&mut analyzer, ngrams, url, frequencies);
    collect_document_stats(&mut analyzer, url, doc_stats);
    
    Ok(())
}

// Fonctions utilitaires
async fn fetch_and_prepare_content(url: &str) -> Result<String, Box<dyn Error>> {
    let mut web_analyzer = WebAnalyzer::new(url);
    Ok(web_analyzer.fetch_and_analyze().await?)
}

fn create_analyzer(content: &str) -> Result<TextAnalyzer, Box<dyn Error>> {
    let mut analyzer = TextAnalyzer::new(content, "stop_words_french.txt")?;
    analyzer.analyze();
    analyzer.remove_special_characters();
    analyzer.clean_word();
    analyzer.count_words();
    Ok(analyzer)
}

fn process_ngrams(
    analyzer: &mut TextAnalyzer,
    ngrams: &[usize],
    url: &str,
    frequencies: &mut HashMap<(String, String), (f64, f64, Vec<String>)>,
) {
    for &n in ngrams {
        analyzer.word_frequency_ngrams(n);
        
        if let Some((freq_map, percent_map)) = analyzer._get_ngram_frequency(n) {
            let gram_type = get_gram_type(n);
            update_frequencies(freq_map, percent_map, gram_type, url, frequencies);
        }
    }
}

fn get_gram_type(n: usize) -> &'static str {
    match n {
        1 => "mot",
        2 => "bigramme",
        3 => "trigramme",
        4 => "quadrigramme",
        5 => "pentagramme",
        _ => "inconnu",
    }
}

fn update_frequencies(
    freq_map: &HashMap<String, usize>,
    percent_map: &HashMap<String, f64>,
    gram_type: &str,
    url: &str,
    frequencies: &mut HashMap<(String, String), (f64, f64, Vec<String>)>,
) {
    for (expr, count) in freq_map {
        let percentage = *percent_map.get(expr).unwrap_or(&0.0);
        let entry = frequencies
            .entry((expr.clone(), gram_type.to_string()))
            .or_insert((0.0, 0.0, Vec::new()));
        
        entry.0 += *count as f64;
        entry.1 += percentage;
        if !entry.2.contains(&url.to_string()) {
            entry.2.push(url.to_string());
        }
    }
}

fn collect_document_stats(analyzer: &mut TextAnalyzer, url: &str, doc_stats: &mut Vec<DocumentStats>) {
    let (total_retained, total_unique, word_count) = analyzer.get_total_stats();
    let avg_word_length = analyzer.average_word_length();
    
    doc_stats.push(DocumentStats {
        url: url.to_string(),
        total_retained,
        total_unique,
        word_count,
        average_word_length: avg_word_length,
    });
}

fn create_url_status(url: &str, success: bool, error: Option<String>) -> UrlStatus {
    UrlStatus {
        url: url.to_string(),
        status: if success { "ok" } else { "ko" }.to_string(),
        error,
    }
}

fn process_frequencies(
    frequencies: HashMap<(String, String), (f64, f64, Vec<String>)>,
    successful_urls: usize,
) -> Vec<FrequencyResult> {
    let mut results: Vec<FrequencyResult> = frequencies
        .into_iter()
        .map(|((expr, gram_type), (count, percentage, sources))| {
            let doc_count = sources.len() as f64;
            FrequencyResult {
                expression: expr,
                gram_type,
                average_occurrences: count / doc_count,
                average_percentage: percentage / doc_count,
                doc_count_percentage: (doc_count / successful_urls as f64) * 100.0,
                sources,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.doc_count_percentage
            .partial_cmp(&a.doc_count_percentage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.average_occurrences.partial_cmp(&a.average_occurrences)
            .unwrap_or(std::cmp::Ordering::Equal))
    });

    results
}

fn serialize_f64_2_decimals<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_f64((x * 100.0).round() / 100.0)
}