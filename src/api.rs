use crate::text_analyzer::TextAnalyzer;
use crate::web_analyzer::WebAnalyzer;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Deserialize)]
pub struct AnalysisRequest {
    urls: Vec<String>,
    ngrams_to_analyze: Option<Vec<usize>>, // optionnel, par défaut [1,2,3]
}

#[derive(Serialize)]
pub struct AnalysisResponse {
    combined_frequencies: Vec<FrequencyResult>,
    analysis_directory: String,
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

// Fonction d'aide pour sérialiser avec 2 décimales
fn serialize_f64_2_decimals<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_f64((x * 100.0).round() / 100.0)
}

#[post("/api/analyze")]
pub async fn analyze_urls(data: web::Json<AnalysisRequest>) -> impl Responder {
    let urls = data.urls.clone();
    let ngrams = data.ngrams_to_analyze.clone().unwrap_or(vec![1, 2, 3]);

    match perform_analysis(urls, ngrams).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn perform_analysis(
    urls: Vec<String>,
    ngrams_to_analyze: Vec<usize>,
) -> Result<AnalysisResponse, Box<dyn Error>> {
    // Créer le dossier d'analyse
    let analysis_dir = create_next_analysis_directory()?;
    let mut all_analyzers = Vec::new();

    // Analyser chaque URL
    for url in &urls {
        let mut web_analyzer = WebAnalyzer::new(url);
        web_analyzer.fetch_and_analyze(&analysis_dir).await?;

        let prefix = web_analyzer.get_domain_prefix();
        let input_file = format!("{}/{}_output.txt", analysis_dir, prefix);

        if std::path::Path::new(&input_file).exists() {
            let mut text_analyzer = TextAnalyzer::new(&input_file, "stop_words_french.txt")?;

            // Effectuer l'analyse complète
            text_analyzer.analyze();
            text_analyzer.longest_sentences(5);
            text_analyzer.punctuation_stats();
            text_analyzer.remove_special_characters();
            text_analyzer.count_words();
            text_analyzer.clean_word();

            for n in &ngrams_to_analyze {
                text_analyzer.word_frequency_ngrams(*n);
            }

            text_analyzer.get_total_stats();
            text_analyzer.average_word_length();

            // Exporter les fréquences individuelles
            text_analyzer.export_frequencies_to_csv(
                &analysis_dir,
                &prefix,
                "frequencies.csv",
                &ngrams_to_analyze,
            )?;

            all_analyzers.push((prefix, text_analyzer));
        }
    }

    // Combiner les résultats
    let combined_path = format!("{}/combined_frequencies.csv", analysis_dir);
    let combined_results = export_combined_results(&all_analyzers, &combined_path)?;

    Ok(AnalysisResponse {
        combined_frequencies: combined_results,
        analysis_directory: analysis_dir,
    })
}

fn create_next_analysis_directory() -> Result<String, Box<dyn Error>> {
    let mut counter = 1;
    loop {
        let dir_name = format!("analyse{}", counter);
        if !std::path::Path::new(&dir_name).exists() {
            std::fs::create_dir(&dir_name)?;
            return Ok(dir_name);
        }
        counter += 1;
    }
}

fn export_combined_results(
    analyzers: &[(String, TextAnalyzer)],
    output_filename: &str,
) -> Result<Vec<FrequencyResult>, Box<dyn Error>> {
    let mut all_frequencies = Vec::new();
    let total_docs = analyzers.len() as f64;

    // Extraire le dossier d'analyse du chemin de sortie
    let analysis_dir = std::path::Path::new(output_filename)
        .parent()
        .ok_or("Impossible d'obtenir le dossier parent")?
        .to_str()
        .ok_or("Chemin invalide")?;

    // Lire chaque fichier CSV individuel
    for (prefix, _) in analyzers {
        let input_filename = format!("{}/{}_frequencies.csv", analysis_dir, prefix);
        let file = match File::open(&input_filename) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let _ = lines.next(); // Sauter l'en-tête

        for line in lines {
            if let Ok(line) = line {
                let parts: Vec<&str> = line.split(';').collect();
                if parts.len() >= 4 {
                    let expr = parts[0].to_string();
                    let gram_type = parts[1].to_string();
                    let count: usize = parts[2].parse().unwrap_or(0);
                    let percentage: f64 = parts[3].trim_end_matches('%').parse().unwrap_or(0.0);

                    all_frequencies.push((expr, gram_type, count, percentage, prefix.clone()));
                }
            }
        }
    }

    // Regrouper par expression et type
    let mut combined_stats: HashMap<(String, String), (usize, f64, Vec<String>)> = HashMap::new();
    for (expr, gram_type, count, percentage, source) in all_frequencies {
        let entry = combined_stats
            .entry((expr, gram_type))
            .or_insert((0, 0.0, Vec::new()));

        entry.0 += count;
        entry.1 += percentage;
        if !entry.2.contains(&source) {
            entry.2.push(source);
        }
    }

    // Convertir en vecteur et trier d'abord par doc_count puis par occurrences
    let mut combined_vec: Vec<_> = combined_stats.into_iter().collect();
    combined_vec.sort_by(|a, b| {
        // Calculer les pourcentages de documents pour a et b
        let doc_count_a = (a.1 .2.len() as f64 / total_docs) * 100.0;
        let doc_count_b = (b.1 .2.len() as f64 / total_docs) * 100.0;

        // Comparer d'abord par doc_count
        match doc_count_b
            .partial_cmp(&doc_count_a)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => {
                // Si égaux, comparer par moyenne d'occurrences
                let avg_a = a.1 .0 as f64 / a.1 .2.len() as f64;
                let avg_b = b.1 .0 as f64 / b.1 .2.len() as f64;
                avg_b
                    .partial_cmp(&avg_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
            other => other,
        }
    });

    let mut results = Vec::new();
    for ((expr, gram_type), (total_count, total_percentage, sources)) in combined_vec {
        let avg_count = total_count as f64 / sources.len() as f64;
        let avg_percentage = total_percentage / sources.len() as f64;
        let doc_count_percentage = (sources.len() as f64 / total_docs) * 100.0;

        results.push(FrequencyResult {
            expression: expr,
            gram_type,
            average_occurrences: avg_count,
            average_percentage: avg_percentage,
            doc_count_percentage,
            sources,
        });
    }

    Ok(results)
}
