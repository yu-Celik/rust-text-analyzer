mod html_analyzer;
mod text_analyzer;
mod web_analyzer;

use crate::text_analyzer::TextAnalyzer;
use crate::web_analyzer::WebAnalyzer;
use std::error::Error;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Demande de l'URL à l'utilisateur
    print!("Veuillez entrer l'URL à analyser (ex: https://www.example.fr/): ");
    io::stdout().flush()?;
    
    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim();

    // Vérification basique de l'URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        println!("Erreur: L'URL doit commencer par 'http://' ou 'https://'");
        return Ok(());
    }

    println!("Analyse de {}", url);
    
    // Analyse de la page web
    let mut web_analyzer = WebAnalyzer::new(url);
    web_analyzer.fetch_and_analyze().await?;

    // Utiliser le préfixe du domaine depuis WebAnalyzer
    let prefix = web_analyzer.get_domain_prefix();
    let input_file = format!("{}_output.txt", prefix);

    // Création et analyse du texte avec TextAnalyzer
    let mut text_analyzer = TextAnalyzer::new(&input_file, "stop_words_french.txt")?;

    // Analyse du texte
    text_analyzer.analyze();
    // Appeler ces méthodes avant remove_special_characters
    text_analyzer.longest_sentences(5);
    text_analyzer.punctuation_stats();

    // Appeler la méthode remove_special_characters ici pour préserver les caractères spéciaux
    text_analyzer.remove_special_characters();
    text_analyzer.count_words();
    text_analyzer.filter_banned_words();

    text_analyzer.word_frequency();
    text_analyzer.word_frequency_ngrams(2);
    text_analyzer.word_frequency_ngrams(3);
    text_analyzer.word_frequency_ngrams(4);
    text_analyzer.word_frequency_ngrams(5);

    text_analyzer.average_word_length();

    // Après l'analyse, utiliser le même préfixe
    text_analyzer.export_frequencies_to_csv(&prefix, "frequencies.csv")?;

    // Affichage des résultats
    println!("\n=== Analyse du texte ===");
    // text_analyzer.print_word_count();
    // text_analyzer.print_ngram_frequency(1); // pour les mots
    // text_analyzer.print_average_word_length();
    // text_analyzer.print_punctuation_stats();

    // Affichage de l'analyse HTML
    web_analyzer.print_analysis();

    Ok(())
}
