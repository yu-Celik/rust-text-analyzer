use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct TextAnalyzer {
    pub content: String,
    word_count: usize,
    word_frequency: HashMap<String, usize>,
    word_frequency_percentage: HashMap<String, f64>,
    pub word_frequency_twograms: HashMap<String, usize>,
    pub word_frequency_twograms_percentage: HashMap<String, f64>,
    pub word_frequency_trigrams: HashMap<String, usize>,
    pub word_frequency_trigrams_percentage: HashMap<String, f64>,
    pub word_frequency_fourgrams: HashMap<String, usize>,
    pub word_frequency_fourgrams_percentage: HashMap<String, f64>,
    pub word_frequency_fivegrams: HashMap<String, usize>,
    pub word_frequency_fivegrams_percentage: HashMap<String, f64>,
    average_word_length: f64,
    longest_sentences: Vec<String>,
    punctuation_stats: HashMap<char, usize>,
    ban_list: HashSet<String>,
    // Statistiques pour chaque type de n-gramme
    pub retained_expressions: HashMap<usize, usize>, // n -> nombre après filtrage
    pub unique_expressions: HashMap<usize, usize>,   // n -> nombre avant filtrage
}

impl TextAnalyzer {
    pub fn new(content: &str, stop_words_path: &str) -> Result<Self, Box<dyn Error>> {
        let stop_words_content = fs::read_to_string(stop_words_path)?;
        let ban_list: HashSet<String> = stop_words_content
            .lines()
            .map(|line| line.to_string())
            .collect();

        Ok(TextAnalyzer {
            content: content.to_string(),
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
            retained_expressions: HashMap::new(),
            unique_expressions: HashMap::new(),
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

    pub fn normalize_apostrophes(&mut self) {
        self.content = self
            .content
            .replace('\u{2019}', "'") // Remplace l'apostrophe typographique par l'apostrophe simple
            .replace('\u{2018}', "'") // Remplace l'apostrophe simple gauche
            .replace('\u{201B}', "'") // Remplace l'apostrophe simple haute
    }

    pub fn clean_word(&mut self) {
        let prefixes_to_remove = [
            "l'", "d'", "n'", "j'", "s'", "c'", "t'", "l'", "d'", "n'", "j'", "s'", "c'", "t'",
        ];

        self.content = self
            .content
            .split_whitespace()
            .map(|word| {
                let mut cleaned = word.to_string();
                for prefix in &prefixes_to_remove {
                    if cleaned.starts_with(prefix) {
                        cleaned = cleaned[prefix.len()..].to_string();
                        break;
                    }
                }
                cleaned
            })
            .collect::<Vec<String>>()
            .join(" ");
    }

    pub fn word_frequency_ngrams(&mut self, n: usize) {
        let words: Vec<&str> = self.content.split_whitespace().collect();
        if words.len() < n {
            return;
        }

        let ngram_map = match n {
            1 => &mut self.word_frequency,
            2 => &mut self.word_frequency_twograms,
            3 => &mut self.word_frequency_trigrams,
            4 => &mut self.word_frequency_fourgrams,
            5 => &mut self.word_frequency_fivegrams,
            _ => {
                println!("Taille de n-gramme non prise en charge : {}", n);
                return;
            }
        };

        ngram_map.clear();
        let mut all_ngrams = HashMap::new();

        // Générer tous les n-grammes possibles
        if n == 1 {
            for word in words {
                *all_ngrams.entry(word.to_string()).or_insert(0) += 1;
            }
        } else {
            for i in 0..=words.len() - n {
                let window = &words[i..i + n];
                let ngram = window.join(" ");
                *all_ngrams.entry(ngram).or_insert(0) += 1;
            }
        }

        // Sauvegarder le nombre d'expressions uniques (avant filtrage)
        self.unique_expressions.insert(n, all_ngrams.len());

        // Appliquer le filtre de la blacklist
        for (ngram, count) in all_ngrams {
            let is_valid = if n == 1 {
                !self.ban_list.contains(&ngram)
            } else {
                let words: Vec<&str> = ngram.split_whitespace().collect();
                !self.ban_list.contains(&words[0].to_string())
                    && !self.ban_list.contains(&words[n - 1].to_string())
            };

            if is_valid {
                *ngram_map.entry(ngram).or_insert(0) = count;
            }
        }

        // Sauvegarder le nombre d'expressions retenues (après filtrage)
        self.retained_expressions.insert(n, ngram_map.len());

        // Calculer les pourcentages
        let total_words = self.word_count as f64;
        match n {
            1 => {
                self.word_frequency_percentage =
                    self.calculate_percentages(&self.word_frequency, total_words)
            }
            2 => {
                self.word_frequency_twograms_percentage =
                    self.calculate_percentages(&self.word_frequency_twograms, total_words)
            }
            3 => {
                self.word_frequency_trigrams_percentage =
                    self.calculate_percentages(&self.word_frequency_trigrams, total_words)
            }
            4 => {
                self.word_frequency_fourgrams_percentage =
                    self.calculate_percentages(&self.word_frequency_fourgrams, total_words)
            }
            5 => {
                self.word_frequency_fivegrams_percentage =
                    self.calculate_percentages(&self.word_frequency_fivegrams, total_words)
            }
            _ => {}
        }
    }

    fn calculate_percentages(
        &self,
        frequency_map: &HashMap<String, usize>,
        total_words: f64,
    ) -> HashMap<String, f64> {
        frequency_map
            .iter()
            .map(|(word, count)| {
                let percentage = (*count as f64 * 100.0) / total_words;
                (word.clone(), percentage)
            })
            .collect()
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

    pub fn filter_banned_words(&mut self) {
        let words: Vec<&str> = self.content.split_whitespace().collect();
        self.content = words
            .into_iter()
            .filter(|word| !self.ban_list.contains(*word))
            .collect::<Vec<&str>>()
            .join(" ");
    }

    pub fn _longest_sentences(&mut self, n: usize) -> &[String] {
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

    pub fn _punctuation_stats(&mut self) -> &HashMap<char, usize> {
        self.punctuation_stats.clear();
        self.content
            .chars()
            .filter(|&c| c.is_ascii_punctuation())
            .for_each(|c| *self.punctuation_stats.entry(c).or_insert(0) += 1);
        &self.punctuation_stats
    }

    pub fn _print_content(&self) {
        println!("Content :\n{}", self.content);
    }

    pub fn _print_average_word_length(&self) {
        println!("Average word length: {:.2}", self.average_word_length);
    }

    pub fn _print_punctuation_stats(&self) {
        for (caractere, frequence) in &self.punctuation_stats {
            println!("The character '{}' appears {} times.", caractere, frequence);
        }
    }

    pub fn _print_longest_sentences(&self) {
        println!("The 3 longest sentences :");
        for (i, sentence) in self.longest_sentences.iter().enumerate() {
            println!("{}. {}", i + 1, sentence);
        }
    }

    pub fn _print_word_count(&self) {
        println!("Word count : {}", self.word_count);
    }

    pub fn _print_ngram_frequency(&self, n: usize) {
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
    

    pub fn _get_ngram_frequency(
        &self,
        n: usize,
    ) -> Option<(&HashMap<String, usize>, &HashMap<String, f64>)> {
        match n {
            1 => Some((&self.word_frequency, &self.word_frequency_percentage)),
            2 => Some((
                &self.word_frequency_twograms,
                &self.word_frequency_twograms_percentage,
            )),
            3 => Some((
                &self.word_frequency_trigrams,
                &self.word_frequency_trigrams_percentage,
            )),
            4 => Some((
                &self.word_frequency_fourgrams,
                &self.word_frequency_fourgrams_percentage,
            )),
            5 => Some((
                &self.word_frequency_fivegrams,
                &self.word_frequency_fivegrams_percentage,
            )),
            _ => None,
        }
    }

    pub fn get_total_stats(&self) -> (usize, usize, usize) {
        // Total des expressions retenues
        let total_retained: usize = self.retained_expressions.values().sum();

        // Total des expressions uniques
        let total_unique: usize = self.unique_expressions.values().sum();

        // Nombre total de mots
        let word_count = self.word_count;

        (total_retained, total_unique, word_count)
    }

    pub fn _count_word_frequency(&self, word: &String) -> usize {
        self.word_frequency.get(word).copied().unwrap_or(0)
    }

}
