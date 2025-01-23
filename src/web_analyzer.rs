use crate::html_analyzer::HtmlAnalyzer;
use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use std::fs;

pub struct WebAnalyzer {
    url: String,
    content: String,
    html_analyzer: HtmlAnalyzer,
    text_content: String, // Nouveau champ pour stocker le texte extrait
}

#[derive(Debug)]
struct PageMetadata {
    title: String,
    description: String,
    keywords: String,
}

impl WebAnalyzer {
    pub fn new(url: &str) -> Self {
        WebAnalyzer {
            url: url.to_string(),
            content: String::new(),
            html_analyzer: HtmlAnalyzer::new(),
            text_content: String::new(),
        }
    }

    pub async fn fetch_and_analyze(&mut self, output_dir: &str) -> Result<(), Box<dyn Error>> {
        // Configuration du client
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .danger_accept_invalid_certs(true)
            .build()?;

        // Tentative de récupération du contenu
        let response = match client.get(&self.url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                println!("Erreur lors de la requête pour {}: {}", self.url, e);
                // Sauvegarder l'URL en échec
                self.save_failed_url(output_dir, "ko")?;
                return Ok(());  // Retourne Ok pour continuer avec les autres URLs
            }
        };

        // Vérification du statut
        if !response.status().is_success() {
            println!("Statut HTTP non-succès pour {}: {}", self.url, response.status());
            self.save_failed_url(output_dir, "ko")?;
            return Ok(());
        }

        // Récupération du contenu
        match response.text().await {
            Ok(text) => {
                self.content = text;
                self.extract_text_content();
                self.save_text_content(output_dir, "output.txt")?;
                self.html_analyzer.analyze(&self.content)?;
            }
            Err(e) => {
                println!("Erreur lors de la lecture du contenu de {}: {}", self.url, e);
                self.save_failed_url(output_dir, "ko")?;
                return Ok(());
            }
        }

        Ok(())
    }

    fn extract_text_content(&mut self) {
        let document = Html::parse_document(&self.content);
        let metadata = self.extract_metadata(&document);
        let mut combined_text = String::new();
    
        // Ajout des métadonnées sans séparation
        combined_text.push_str(&metadata.title);
        combined_text.push_str(" ");
        combined_text.push_str(&metadata.description);
        combined_text.push_str(" ");
        combined_text.push_str(&metadata.keywords);
        combined_text.push_str(" ");
    
        // Ajout du contenu du body sans séparation
        if let Some(body_content) = self.extract_body_content(&document) {
            combined_text.push_str(&body_content);
        }
    
        self.text_content = combined_text;
    }

    fn extract_metadata(&self, document: &Html) -> PageMetadata {
        let title_selector = Selector::parse("title").unwrap();
        let meta_description = Selector::parse("meta[name='description']").unwrap();
        let meta_keywords = Selector::parse("meta[name='keywords']").unwrap();

        PageMetadata {
            title: document
                .select(&title_selector)
                .next()
                .map(|t| t.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
            description: document
                .select(&meta_description)
                .next()
                .and_then(|m| m.value().attr("content"))
                .unwrap_or_default()
                .to_string(),
            keywords: document
                .select(&meta_keywords)
                .next()
                .and_then(|m| m.value().attr("content"))
                .unwrap_or_default()
                .replace(",", ", ")
                .to_string(),
        }
    }

    fn extract_body_content(&self, document: &Html) -> Option<String> {
        let body_selector = Selector::parse("body").unwrap();
        let script_selector = Selector::parse("script").unwrap();
        let style_selector = Selector::parse("style").unwrap();
    
        document.select(&body_selector).next().map(|body| {
            let mut content = body.html();
    
            // Suppression des scripts et styles
            for script in document.select(&script_selector) {
                content = content.replace(&script.html(), "");
            }
            for style in document.select(&style_selector) {
                content = content.replace(&style.html(), "");
            }
    
            // Extraction du texte propre
            let clean_document = Html::parse_document(&content);
            let mut combined_text = String::new();
            
            for node in clean_document.root_element().descendants() {
                if let Some(text_node) = node.value().as_text() {
                    combined_text.push_str(text_node);
                    combined_text.push(' ');
                }
            }
            
            combined_text
                .replace('\n', " ")
                .replace('\t', " ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    pub fn get_domain_prefix(&self) -> String {
        let without_protocol = self.url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("www.")
            .trim_end_matches('/');
    
        // Prendre tout le chemin, pas seulement le domaine
        let full_path = without_protocol.replace('/', "_");
    
        // Nettoyer le chemin pour le nom de fichier
        full_path
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
    }
    fn save_text_content(&self, output_dir: &str, filename: &str) -> Result<(), Box<dyn Error>> {
        let prefix = self.get_domain_prefix();
        let full_filename = format!("{}/{}_{}", output_dir, prefix, filename);
        fs::write(full_filename, &self.text_content)?;
        Ok(())
    }

    pub fn _get_text_content(&self) -> &str {
        &self.text_content
    }

    pub fn _print_analysis(&self) {
        println!("\n=== Analyse de la page {} ===\n", self.url);
        self.html_analyzer.print_stats();
    }

    // Nouvelle méthode pour sauvegarder les URLs en échec
    fn save_failed_url(&self, output_dir: &str, status: &str) -> Result<(), Box<dyn Error>> {
        let prefix = self.get_domain_prefix();
        let failed_file = format!("{}/{}_failed.txt", output_dir, prefix);
        std::fs::write(failed_file, format!("{}\t{}", self.url, status))?;
        Ok(())
    }
}
