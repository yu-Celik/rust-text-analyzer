mod html_analyzer;
mod search_crawler;
mod tests;
mod text_analyzer;
mod web_analyzer;

use crate::text_analyzer::TextAnalyzer;
use crate::web_analyzer::WebAnalyzer;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engines = vec!["google"];
    let phrase = "seo";
    let start = 0;
    let num_to_crawl = 10;
    let existing_urls_length = 0;
    let language = "fr";
    let blacklist = search_crawler::get_blacklist();

    // Créer le dossier d'analyse
    let analysis_dir = create_next_analysis_directory()?;
    let mut urls = Vec::new();

    // urls.push("https://www.larousse.fr/dictionnaires/francais/professeur/64155");
    // urls.push("https://dictionnaire.lerobert.com/definition/professeur");
    // urls.push("https://fr.wikipedia.org/wiki/Enseignant");
    // urls.push("https://fr.wikipedia.org/wiki/Professeur");
    // urls.push("https://www.dictionnaire-academie.fr/article/A9P4464");
    // urls.push("https://www.cnrtl.fr/definition/professeur");
    // urls.push("https://fr.wiktionary.org/wiki/professeur");
    // urls.push("https://www.letudiant.fr/etudes/devenir-professeur-mode-demploi.html");
    // urls.push("https://www.lalanguefrancaise.com/dictionnaire/definition/professeur");
    // urls.push("https://www.hellowork.com/fr-fr/metiers/professeur.html");

    // // Récupérer les URLs via l'API
    for engine in engines {
        println!("\nRecherche sur {}", engine);

        match search_crawler::call_search_engine_crawler(
            engine,
            phrase,
            start,
            num_to_crawl,
            existing_urls_length,
            language,
            blacklist.clone()
        ).await {
            Ok(Some(response)) => {
                if response.success {
                    for result in &response.data {
                        // println!("\nURL: {}", result.url);
                        urls.push(result.url.clone());
                    }
                }
            },
            Ok(None) => println!("Aucun résultat pour {}", engine),
            Err(e) => println!("Erreur pour {}: {}", engine, e),
        }
    }

    let mut all_analyzers = Vec::new();
    let ngrams_to_analyze = vec![1, 2, 3];

    // Analyser chaque URL récupérée
    for url in &urls {
        let mut web_analyzer = WebAnalyzer::new(url);

        web_analyzer.fetch_and_analyze(&analysis_dir).await?;

        let prefix = web_analyzer.get_domain_prefix();
        let input_file = format!("{}/{}_output.txt", analysis_dir, prefix);

        // Attendre que le fichier existe
        if std::path::Path::new(&input_file).exists() {
            let mut text_analyzer = TextAnalyzer::new(&input_file, "stop_words_french.txt")?;
            text_analyzer.analyze();
            // Appeler ces méthodes avant remove_special_characters
            text_analyzer.longest_sentences(5);
            text_analyzer.punctuation_stats();

            // Appeler la méthode remove_special_characters ici pour préserver les caractères spéciaux
            text_analyzer.remove_special_characters();
            text_analyzer.count_words();
            text_analyzer.clean_word();

            // Après l'analyse des n-grammes
            text_analyzer.word_frequency_ngrams(2);
            // text_analyzer.word_frequency_ngrams(3);


            text_analyzer.filter_banned_words();

            text_analyzer.word_frequency_ngrams(1);

            text_analyzer.get_total_stats();


            // text_analyzer._print_ngram_frequency(2);
            // text_analyzer._print_ngram_frequency(3);

            text_analyzer.average_word_length();

            // Sauvegarder les résultats individuels
            text_analyzer.export_frequencies_to_csv(
                &analysis_dir,
                &prefix,
                "frequencies.csv",
                &ngrams_to_analyze,
            )?;

            // Stocker l'analyseur pour la fusion finale
            all_analyzers.push((prefix, text_analyzer));

            // Afficher l'analyse HTML
            // web_analyzer.print_analysis();
        } else {
            println!("Erreur: Fichier {} non trouvé", input_file);
        }
    }

    // Spécifier les n-grams désirés (par exemple 1, 2 et 3)

    // Fusionner et exporter les résultats combinés
    if !all_analyzers.is_empty() {
        let combined_path = format!("{}/combined_frequencies.csv", analysis_dir);
        export_combined_results(&all_analyzers, &combined_path, &ngrams_to_analyze)?;
    }

    Ok(())
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
    ngrams: &[usize],
) -> Result<(), Box<dyn Error>> {
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

    // Écrire le fichier combiné
    let file = File::create(output_filename)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(&[0xEF, 0xBB, 0xBF])?;
    writeln!(
        writer,
        "expression;type;moyenne_occurrences;moyenne_pourcentage;doc_count%;sources"
    )?;

    for ((expr, gram_type), (total_count, total_percentage, sources)) in combined_vec {
        let avg_count = total_count as f64 / sources.len() as f64;
        let avg_percentage = total_percentage / sources.len() as f64;
        let doc_count_percentage = (sources.len() as f64 / total_docs) * 100.0;
        let sources_str = sources.join(", ");

        writeln!(
            writer,
            "{};{};{:.2};{:.2}%;{:.2}%;{}",
            expr, gram_type, avg_count, avg_percentage, doc_count_percentage, sources_str
        )?;
    }

    Ok(())
}


// https://www.larousse.fr/dictionnaires/francais/professeur/64155
// https://dictionnaire.lerobert.com/definition/professeur
// https://fr.wikipedia.org/wiki/Enseignant
// https://fr.wikipedia.org/wiki/Professeur
// https://www.dictionnaire-academie.fr/article/A9P4464
// https://www.cnrtl.fr/definition/professeur
// https://fr.wiktionary.org/wiki/professeur
// https://www.letudiant.fr/etudes/devenir-professeur-mode-demploi.html
// https://www.lalanguefrancaise.com/dictionnaire/definition/professeur
// https://www.hellowork.com/fr-fr/metiers/professeur.html