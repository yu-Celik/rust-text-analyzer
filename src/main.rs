use rust_text_analyzer::TextAnalyzer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file_path> <stop_words_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let stop_words_path = &args[2];

    match TextAnalyzer::new(file_path, stop_words_path) {
        Ok(mut analyzer) => {
            analyzer.analyze();
            // Appeler ces méthodes avant remove_special_characters
            analyzer.longest_sentences();
            analyzer.punctuation_stats();

            // Appeler la méthode remove_special_characters ici pour préserver les caractères spéciaux
            analyzer.remove_special_characters();
            analyzer.count_words();
            analyzer.filter_banned_words();

            analyzer.word_frequency();
            analyzer.word_frequency_ngrams(2);
            analyzer.word_frequency_ngrams(3);
            analyzer.word_frequency_ngrams(4);
            analyzer.word_frequency_ngrams(5);

            analyzer.average_word_length();

            
            // analyzer.print_word_frequency();
            // analyzer.print_word_frequency_twograms();
            // analyzer.print_punctuation_stats();
            // analyzer.print_word_count();
            // analyzer.print_longest_sentences();
            analyzer.print_average_word_length();
        }
        Err(e) => eprintln!("Error reading file {} : {}", file_path, e),
    }
}
