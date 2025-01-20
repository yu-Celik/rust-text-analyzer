#[cfg(test)]
mod tests {
    use crate::TextAnalyzer;

    #[test]
    fn test_word_count() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("This is a test sentence.");
        analyzer.remove_special_characters();
        assert_eq!(analyzer.count_words(), 5);
    }

    #[test]
    fn test_remove_special_characters() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("Hello, world! 1910 #$% αβγ ? you're low-level high_score");
        analyzer.remove_special_characters();
        assert_eq!(
            analyzer.content,
            "Hello world 1910 αβγ you're low-level high_score"
        );
    }

    #[test]
    fn test_remove_special_characters_with_punctuation() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("Bonjour! Comment allez-vous? C'est une belle journée.");
        analyzer.remove_special_characters();
        assert_eq!(
            analyzer.content,
            "Bonjour Comment allez-vous C'est une belle journée"
        );
    }

    #[test]
    fn test_remove_special_characters_with_numbers() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("Le prix est de 99,99€ pour 3 articles.");
        analyzer.remove_special_characters();
        assert_eq!(analyzer.content, "Le prix est de 99,99 pour 3 articles");
    }

    #[test]
    fn test_remove_special_characters_with_symbols() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("email@example.com & 100% sûr!");
        analyzer.remove_special_characters();
        assert_eq!(analyzer.content, "email@example.com 100 sûr");
    }

    #[test]
    fn test_word_frequency() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content =
            String::from("The quick brown fox jumps over the lazy dog. The fox is quick.");
        analyzer.analyze();
        analyzer.remove_special_characters();
        analyzer.count_words();
        // println!("{}", analyzer.content);
        analyzer.word_frequency();
        assert_eq!(analyzer.count_word_frequency(&"the".to_string()), 3);
        assert_eq!(analyzer.count_word_frequency(&"quick".to_string()), 2);
        assert_eq!(analyzer.count_word_frequency(&"fox".to_string()), 2);
        assert_eq!(analyzer.count_word_frequency(&"jumps".to_string()), 1);
        assert_eq!(analyzer.count_word_frequency(&"nonexistent".to_string()), 0);
    }
    #[test]
    fn test_word_frequency_french() {
        let mut analyzer =
            TextAnalyzer::new("programming_tips.txt", "stop_words_french.txt").unwrap();
        analyzer.content = String::from("Le petit chat est sur le tapis. Le chat est mignon.");
        analyzer.analyze();
        analyzer.remove_special_characters();
        analyzer.count_words();
        analyzer.word_frequency();
        assert_eq!(analyzer.count_word_frequency(&"le".to_string()), 3);
        assert_eq!(analyzer.count_word_frequency(&"chat".to_string()), 2);
        assert_eq!(analyzer.count_word_frequency(&"est".to_string()), 2);
        assert_eq!(analyzer.count_word_frequency(&"petit".to_string()), 1);
        assert_eq!(analyzer.count_word_frequency(&"inexistant".to_string()), 0);
    }

    #[test]
    fn test_average_word_length_french() {
        let mut analyzer =
            TextAnalyzer::new("programming_tips.txt", "stop_words_french.txt").unwrap();
        analyzer.content = String::from("Le chat noir saute.");
        analyzer.remove_special_characters();
        analyzer.count_words();
        analyzer.word_frequency();
        assert_eq!(analyzer.average_word_length(), 3.75);
    }

    #[test]
    fn test_remove_special_characters_french() {
        let mut analyzer =
            TextAnalyzer::new("programming_tips.txt", "stop_words_french.txt").unwrap();
        analyzer.content = String::from("Bonjour, l'ami! Comment ça va? L'été est beau.");
        analyzer.remove_special_characters();
        assert_eq!(
            analyzer.content,
            "Bonjour l'ami Comment ça va L'été est beau"
        );
    }

    #[test]
    fn test_longest_sentences_french() {
        let mut analyzer =
            TextAnalyzer::new("programming_tips.txt", "stop_words_french.txt").unwrap();
        analyzer.content = String::from("C'est une courte phrase. Voici une phrase un peu plus longue. Cette phrase est la plus longue de toutes.");
        let longest = analyzer.longest_sentences(3);
        assert_eq!(longest[0], "Cette phrase est la plus longue de toutes.");
        assert_eq!(longest[1], "Voici une phrase un peu plus longue.");
        assert_eq!(longest[2], "C'est une courte phrase.");
    }

    #[test]
    fn test_punctuation_stats_french() {
        let mut analyzer =
            TextAnalyzer::new("programming_tips.txt", "stop_words_french.txt").unwrap();
        analyzer.content = String::from("Bonjour! Comment allez-vous? J'espère que tout va bien.");
        let stats = analyzer.punctuation_stats();
        assert_eq!(*stats.get(&'!').unwrap_or(&0), 1);
        assert_eq!(*stats.get(&'?').unwrap_or(&0), 1);
        assert_eq!(*stats.get(&'\'').unwrap_or(&0), 1);
        assert_eq!(*stats.get(&'.').unwrap_or(&0), 1);
    }

    #[test]
    fn test_average_word_length() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("The quick brown fox jumps.");
        analyzer.remove_special_characters();
        analyzer.count_words();
        analyzer.word_frequency();
        assert_eq!(analyzer.average_word_length(), 4.2);
    }

    #[test]
    fn test_empty_content() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::new();
        analyzer.remove_special_characters();
        assert_eq!(analyzer.content, "");
        assert_eq!(analyzer.count_words(), 0);
        assert_eq!(analyzer.average_word_length(), 0.0);
    }

    #[test]
    fn test_non_latin_characters() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("Привет мир");
        analyzer.remove_special_characters();
        assert_eq!(analyzer.content, "Привет мир");
        assert_eq!(analyzer.count_words(), 2);
    }

    #[test]
    fn test_content_immutable_after_word_count() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        let original_content = String::from("The quick brown fox jumps over the lazy dog.");
        analyzer.content = original_content.clone();
        analyzer.count_words();
        assert_eq!(analyzer.content, original_content);
    }

    #[test]
    fn test_generate_ngrams() {
        let mut analyzer = TextAnalyzer::new("analyser.txt", "stop_words_english.txt").unwrap();
        analyzer.content = String::from("hello world hello");
        analyzer.word_frequency_ngrams(2);
        assert_eq!(
            analyzer.word_frequency_twograms.get("hello world"),
            Some(&1)
        );
        assert_eq!(
            analyzer.word_frequency_twograms.get("world hello"),
            Some(&1)
        );
    }

    #[test]
    fn test_generate_ngrams_french() {
        let mut analyzer =
            TextAnalyzer::new("programming_tips.txt", "stop_words_french.txt").unwrap();
        analyzer.content = String::from("bonjour le monde bonjour");
        analyzer.word_frequency_ngrams(2);
        assert_eq!(analyzer.word_frequency_twograms.get("bonjour le"), Some(&1));
        assert_eq!(analyzer.word_frequency_twograms.get("le monde"), Some(&1));
        assert_eq!(
            analyzer.word_frequency_twograms.get("monde bonjour"),
            Some(&1)
        );
    }
}
