use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
mod tests;
#[derive(Debug)]
pub struct TextAnalyzer {
    content: String,
    word_count: usize,
    word_frequency: HashMap<String, usize>,
    word_frequency_twograms: HashMap<String, usize>,
    word_frequency_trigrams: HashMap<String, usize>,
    word_frequency_fourgrams: HashMap<String, usize>,
    word_frequency_fivegrams: HashMap<String, usize>,
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
            word_frequency_twograms: HashMap::new(),
            word_frequency_trigrams: HashMap::new(),
            word_frequency_fourgrams: HashMap::new(),
            word_frequency_fivegrams: HashMap::new(),
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
            .filter_map(|word| {
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
    }
    pub fn word_frequency_ngrams(&mut self, n: usize) {
        let words: Vec<&str> = self.content.split_whitespace().collect();
        if words.len() < n {
            return;
        }

        let ngram_map = match n {
            2 => &mut self.word_frequency_twograms,
            3 => &mut self.word_frequency_trigrams,
            4 => &mut self.word_frequency_fourgrams,
            5 => &mut self.word_frequency_fivegrams,
            _ => {
                println!("Taille de n-gramme non prise en charge : {}", n);
                panic!("Taille de n-gramme non prise en charge");
            }
        };

        ngram_map.clear(); // Effacement des fréquences existantes

        for window in words.windows(n) {
            let ngram = window.join(" ");
            *ngram_map.entry(ngram.clone()).or_insert(0) += 1;
        }
    }

    pub fn count_words(&mut self) -> usize {
        let count = self.content.split_whitespace().count();
        self.word_count = count;
        count
    }

    pub fn count_word_frequency(&self, word: &String) -> usize {
        *self.word_frequency.get(word).unwrap_or(&0)
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

    pub fn longest_sentences(&mut self) -> &[String] {
        let sentences: Vec<&str> = self.content.split_inclusive(&['.', '!', '?']).collect();
        self.longest_sentences = sentences
            .into_iter()
            .map(|s| s.trim().split_whitespace().collect::<Vec<&str>>().join(" "))
            .filter(|s| !s.is_empty() && s.split_whitespace().count() > 3)
            .collect();

        self.longest_sentences.sort_by(|a, b| b.len().cmp(&a.len()));
        self.longest_sentences.truncate(3);
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

    pub fn print_word_frequency(&self) {
        println!("Word frequency :");
        for (word, count) in &self.word_frequency {
            println!("{}: {}", word, count);
        }
    }

    pub fn print_word_frequency_twograms(&self) {
        println!("Twogram frequency :");
        for (twogram, count) in &self.word_frequency_twograms {
            println!("{}: {}", twogram, count);
        }
    }
    pub fn print_word_frequency_trigrams(&self) {
        println!("Trigram frequency :");
        for (trigram, count) in &self.word_frequency_trigrams {
            println!("{}: {}", trigram, count);
        }
    }
    pub fn print_word_frequency_fourgrams(&self) {
        println!("Fourgram frequency :");
        for (fourgram, count) in &self.word_frequency_fourgrams {
            println!("{}: {}", fourgram, count);
        }
    }
    pub fn print_word_frequency_fivegrams(&self) {
        println!("Fivegram frequency :");
        for (fivegram, count) in &self.word_frequency_fivegrams {
            println!("{}: {}", fivegram, count);
        }
    }
}
