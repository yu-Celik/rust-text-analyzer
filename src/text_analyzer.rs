use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct TextAnalyzer {
    content: String,
    word_count: usize,
    word_frequency: HashMap<String, usize>,
    word_frequency_percentage: HashMap<String, f64>,
    word_frequency_twograms: HashMap<String, usize>,
    word_frequency_twograms_percentage: HashMap<String, f64>,
    word_frequency_trigrams: HashMap<String, usize>,
    word_frequency_trigrams_percentage: HashMap<String, f64>,
    word_frequency_fourgrams: HashMap<String, usize>,
    word_frequency_fourgrams_percentage: HashMap<String, f64>,
    word_frequency_fivegrams: HashMap<String, usize>,
    word_frequency_fivegrams_percentage: HashMap<String, f64>,
    average_word_length: f64,
    longest_sentences: Vec<String>,
    punctuation_stats: HashMap<char, usize>,
    ban_list: HashSet<String>,
}

impl TextAnalyzer {
    pub fn new(file_path: &str, stop_words_path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        let stop_words_content = fs::read_to_string(stop_words_path)?;
        let ban_list: HashSet<String> = stop_words_content
            .lines()
            .map(|line| line.to_string())
            .collect();

        Ok(TextAnalyzer {
            content,
            word_count: 0,
            word_frequency: HashMap::new(),
            word_frequency_percentage: HashMap::new(),
            word_frequency_twograms: HashMap::new(),
            word_frequency_twograms_percentage: HashMap::new(),
            word_frequency_trigrams: HashMap::new(),
            word_frequency_trigrams_percentage: HashMap::new(),
            word_frequency_fourgrams: HashMap::new(),
            word_frequency_fourgrams_percentage: HashMap::new(),
            word_frequency_fivegrams: HashMap::new(),
            word_frequency_fivegrams_percentage: HashMap::new(),
            average_word_length: 0.0,
            longest_sentences: vec![],
            punctuation_stats: HashMap::new(),
            ban_list,
        })
    }

    pub fn analyze(&mut self) {
        self.content = self.content.to_lowercase();
    }

    /// Removes special characters from the analyzer's content.
    ///
    /// This function iterates over each word in the content, filtering characters
    /// to keep only those that are alphanumeric or surrounded by alphanumeric characters.
    /// This allows preserving characters such as apostrophes in contractions or hyphens in compound words.
    ///
    /// # Examples
    ///
    /// If the content is "Hello, world! 1910 #$% αβγ ? you're low-level high_score",
    /// after calling this function, it will be transformed into
    /// "Hello world 1910 αβγ you're low-level high_score".
    ///
    /// # Returns
    ///
    /// This function does not return anything. It directly modifies `self.content`
    /// by replacing the original content with the filtered version.
    pub fn remove_special_characters(&mut self) {
        self.content = self
            .content
            .split_whitespace()
            .filter_map(|word: &str| {
                let filtered: String = word
                    .chars()
                    .enumerate()
                    .filter(|&(i, c)| {
                        c.is_alphanumeric()
                            || (i > 0
                                && i < word.len() - 1
                                && word
                                    .chars()
                                    .nth(i - 1)
                                    .map_or(false, |prev| prev.is_alphanumeric())
                                && word
                                    .chars()
                                    .nth(i + 1)
                                    .map_or(false, |next| next.is_alphanumeric()))
                    })
                    .map(|(_, c)| c)
                    .collect();
                if filtered.is_empty() {
                    None
                } else {
                    Some(filtered)
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
    }

    pub fn word_frequency(&mut self) {
        for word in self.content.split_whitespace() {
            *self.word_frequency.entry(word.to_string()).or_insert(0) += 1;
        }

        // Calcul des pourcentages
        let total_words = self.word_count as f64;
        for (word, count) in &self.word_frequency {
            let percentage = (*count as f64 * 100.0) / total_words;
            self.word_frequency_percentage
                .insert(word.clone(), percentage);
        }
    }

    pub fn word_frequency_ngrams(&mut self, n: usize) {
        let words: Vec<&str> = self.content.split_whitespace().collect();
        if words.len() < n {
            return;
        }

        let (ngram_map, percentage_map) = match n {
            2 => (
                &mut self.word_frequency_twograms,
                &mut self.word_frequency_twograms_percentage,
            ),
            3 => (
                &mut self.word_frequency_trigrams,
                &mut self.word_frequency_trigrams_percentage,
            ),
            4 => (
                &mut self.word_frequency_fourgrams,
                &mut self.word_frequency_fourgrams_percentage,
            ),
            5 => (
                &mut self.word_frequency_fivegrams,
                &mut self.word_frequency_fivegrams_percentage,
            ),
            _ => {
                println!("Taille de n-gramme non prise en charge : {}", n);
                return;
            }
        };

        ngram_map.clear();
        percentage_map.clear();

        // Calcul des fréquences
        for window in words.windows(n) {
            let ngram = window.join(" ");
            *ngram_map.entry(ngram).or_insert(0) += 1;
        }

        // Calcul des pourcentages
        let total_words = self.word_count as f64;
        for (ngram, count) in ngram_map.iter() {
            let percentage = (*count as f64 * 100.0) / total_words;
            percentage_map.insert(ngram.clone(), percentage);
        }
    }

    pub fn count_words(&mut self) -> usize {
        let count = self.content.split_whitespace().count();
        self.word_count = count;
        count
    }

    pub fn average_word_length(&mut self) -> f64 {
        let mut total_length = 0.0;

        let total_words = self.word_count;

        self.average_word_length = 0.0;
        for (word, &frequency) in &self.word_frequency {
            total_length += word.len() as f64 * frequency as f64;
        }
        self.average_word_length = if total_words > 0 {
            total_length / total_words as f64
        } else {
            0.0
        };
        self.average_word_length
    }

    pub fn longest_sentences(&mut self, n: usize) -> &[String] {
        let sentences: Vec<&str> = self.content.split_inclusive(&['.', '!', '?']).collect();
        self.longest_sentences = sentences
            .into_iter()
            .map(|s| s.trim().split_whitespace().collect::<Vec<&str>>().join(" "))
            .filter(|s| !s.is_empty() && s.split_whitespace().count() > 3)
            .collect();

        self.longest_sentences.sort_by(|a, b| b.len().cmp(&a.len()));
        self.longest_sentences.truncate(n);
        &self.longest_sentences
    }

    pub fn punctuation_stats(&mut self) -> &HashMap<char, usize> {
        self.punctuation_stats.clear();
        self.content
            .chars()
            .filter(|&c| c.is_ascii_punctuation())
            .for_each(|c| *self.punctuation_stats.entry(c).or_insert(0) += 1);
        &self.punctuation_stats
    }

    pub fn filter_banned_words(&mut self) {
        let words: Vec<&str> = self.content.split_whitespace().collect();
        self.content = words
            .into_iter()
            .filter(|word| !self.ban_list.contains(*word))
            .collect::<Vec<&str>>()
            .join(" ");
    }

    pub fn print_content(&self) {
        println!("Content :\n{}", self.content);
    }

    pub fn print_average_word_length(&self) {
        println!("Average word length: {:.2}", self.average_word_length);
    }

    pub fn print_punctuation_stats(&self) {
        for (caractere, frequence) in &self.punctuation_stats {
            println!("The character '{}' appears {} times.", caractere, frequence);
        }
    }

    pub fn print_longest_sentences(&self) {
        println!("The 3 longest sentences :");
        for (i, sentence) in self.longest_sentences.iter().enumerate() {
            println!("{}. {}", i + 1, sentence);
        }
    }

    pub fn print_word_count(&self) {
        println!("Word count : {}", self.word_count);
    }

    pub fn print_ngram_frequency(&self, n: usize) {
        let (ngram_map, percentage_map) = match n {
            1 => (&self.word_frequency, &self.word_frequency_percentage),
            2 => (
                &self.word_frequency_twograms,
                &self.word_frequency_twograms_percentage,
            ),
            3 => (
                &self.word_frequency_trigrams,
                &self.word_frequency_trigrams_percentage,
            ),
            4 => (
                &self.word_frequency_fourgrams,
                &self.word_frequency_fourgrams_percentage,
            ),
            5 => (
                &self.word_frequency_fivegrams,
                &self.word_frequency_fivegrams_percentage,
            ),
            _ => {
                println!("Taille de n-gramme non supportée: {}", n);
                return;
            }
        };

        let gram_name = match n {
            1 => "Word",
            2 => "Twogram",
            3 => "Trigram",
            4 => "Fourgram",
            5 => "Fivegram",
            _ => unreachable!(),
        };

        println!("\n{} frequency:", gram_name);

        // Préparation des données triées
        let mut frequency_vec: Vec<_> = ngram_map.iter().collect();
        frequency_vec.sort_by(|a, b| b.1.cmp(a.1));

        // Affichage des occurrences
        println!("Occurrences:");
        for (gram, count) in &frequency_vec {
            println!("{}: {}", gram, count);
        }

        // Affichage des pourcentages
        println!("\nPourcentages:");
        for (gram, _) in frequency_vec {
            if let Some(percentage) = percentage_map.get(gram) {
                println!("{}: {:.2}%", gram, percentage);
            }
        }
    }

    // Méthode pour créer un nom de fichier avec préfixe
    fn create_filename(&self, prefix: &str, filename: &str) -> String {
        format!("{}_{}", prefix, filename)
    }

    pub fn export_frequencies_to_csv(
        &self,
        prefix: &str,
        filename: &str,
    ) -> Result<(), Box<dyn Error>> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let full_filename = self.create_filename(prefix, filename);
        let file = File::create(full_filename)?;
        let mut writer = BufWriter::new(file);

        // Écrire le BOM UTF-8 pour Excel
        writer.write_all(&[0xEF, 0xBB, 0xBF])?;

        // En-tête avec séparateur point-virgule
        writeln!(writer, "mot;occurrences;pourcentage")?;

        // Trier les mots par fréquence
        let mut frequency_vec: Vec<_> = self.word_frequency.iter().collect();
        frequency_vec.sort_by(|a, b| b.1.cmp(a.1));

        // Écrire les données avec séparation par point-virgule
        for (word, count) in frequency_vec {
            let percentage = self.word_frequency_percentage.get(word).unwrap_or(&0.0);
            writeln!(writer, "{};{};{:.2}%", word, count, percentage)?;
        }

        Ok(())
    }
}
