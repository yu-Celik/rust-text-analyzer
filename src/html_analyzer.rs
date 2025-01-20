use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;

/// Structure dédiée à l'analyse de contenu HTML
#[derive(Debug)]
pub struct HtmlAnalyzer {
    html_content: Option<Html>,
    tag_frequency: HashMap<String, usize>,
    semantic_structure: HashMap<String, Vec<String>>,
    accessibility_issues: Vec<String>,
}

impl HtmlAnalyzer {
    /// Crée une nouvelle instance de HtmlAnalyzer
    pub fn new() -> Self {
        HtmlAnalyzer {
            html_content: None,
            tag_frequency: HashMap::new(),
            semantic_structure: HashMap::new(),
            accessibility_issues: Vec::new(),
        }
    }

    /// Analyse le contenu HTML fourni
    pub fn analyze(&mut self, content: &str) -> Result<(), Box<dyn Error>> {
        let document = Html::parse_document(content);
        self.html_content = Some(document.clone());
        
        self.analyze_tags(&document);
        self.check_accessibility(&document);
        self.analyze_semantic_structure(&document);
        
        Ok(())
    }

    /// Analyse la fréquence des balises HTML
    fn analyze_tags(&mut self, document: &Html) {
        let all_elements = Selector::parse("*").unwrap();
        
        for element in document.select(&all_elements) {
            let tag_name = element.value().name().to_string();
            *self.tag_frequency.entry(tag_name).or_insert(0) += 1;
        }
    }

    /// Vérifie les problèmes d'accessibilité courants
    fn check_accessibility(&mut self, document: &Html) {
        // Vérification des images sans alt
        if let Ok(img_selector) = Selector::parse("img") {
            for img in document.select(&img_selector) {
                if img.value().attr("alt").is_none() {
                    self.accessibility_issues.push(
                        "Image sans attribut alt détectée".to_string()
                    );
                }
            }
        }

        // Vérification des formulaires sans labels
        if let Ok(input_selector) = Selector::parse("input") {
            for input in document.select(&input_selector) {
                if input.value().attr("id").is_none() {
                    self.accessibility_issues.push(
                        "Champ de formulaire sans ID détecté".to_string()
                    );
                }
            }
        }

        // Vérification des niveaux de titres
        self.check_heading_hierarchy(document);
    }

    /// Vérifie la hiérarchie des titres
    fn check_heading_hierarchy(&mut self, document: &Html) {
        let mut previous_level = 0;
        for level in 1..=6 {
            if let Ok(heading_selector) = Selector::parse(&format!("h{}", level)) {
                let count = document.select(&heading_selector).count();
                if count > 0 {
                    if level > previous_level + 1 && previous_level != 0 {
                        self.accessibility_issues.push(
                            format!("Saut dans la hiérarchie des titres: h{} après h{}", 
                                level, previous_level)
                        );
                    }
                    previous_level = level;
                }
            }
        }
    }

    /// Analyse la structure sémantique du document
    fn analyze_semantic_structure(&mut self, document: &Html) {
        let semantic_tags = vec![
            "header", "nav", "main", "article", "section", 
            "aside", "footer", "h1", "h2", "h3", "h4", "h5", "h6"
        ];

        for tag in semantic_tags {
            if let Ok(selector) = Selector::parse(tag) {
                let elements: Vec<String> = document
                    .select(&selector)
                    .map(|el| el.inner_html())
                    .collect();
                
                if !elements.is_empty() {
                    self.semantic_structure.insert(tag.to_string(), elements);
                }
            }
        }
    }

    /// Retourne les statistiques des balises
    pub fn _get_tag_stats(&self) -> &HashMap<String, usize> {
        &self.tag_frequency
    }

    /// Retourne la structure sémantique
    pub fn _get_semantic_structure(&self) -> &HashMap<String, Vec<String>> {
        &self.semantic_structure
    }

    /// Retourne les problèmes d'accessibilité
    pub fn _get_accessibility_issues(&self) -> &Vec<String> {
        &self.accessibility_issues
    }

    /// Affiche les statistiques HTML
    pub fn print_stats(&self) {
        println!("\n=== Statistiques HTML ===");
        
        println!("\nFréquence des balises:");
        for (tag, count) in &self.tag_frequency {
            println!("{}: {}", tag, count);
        }

        println!("\nStructure sémantique:");
        for (tag, elements) in &self.semantic_structure {
            println!("{}: {} éléments trouvés", tag, elements.len());
        }

        println!("\nProblèmes d'accessibilité:");
        if self.accessibility_issues.is_empty() {
            println!("Aucun problème d'accessibilité détecté");
        } else {
            for issue in &self.accessibility_issues {
                println!("- {}", issue);
            }
        }
    }
} 