use reqwest;
use scraper::{Html, Selector};
use std::error::Error;

pub struct WebAnalyzer {
    url: String,
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
        }
    }

    pub async fn fetch_and_analyze(&mut self) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .danger_accept_invalid_certs(true)
            .build()?;

        let response = client.get(&self.url).send().await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);

        // Extraire le texte comme dans extract_text_content
        let metadata = self.extract_metadata(&document);
        let mut combined_text = String::new();

        combined_text.push_str(&metadata.title);
        combined_text.push_str(" ");
        combined_text.push_str(&metadata.description);
        combined_text.push_str(" ");
        combined_text.push_str(&metadata.keywords);
        combined_text.push_str(" ");

        if let Some(body_content) = self.extract_body_content(&document) {
            combined_text.push_str(&body_content);
        }

        Ok(combined_text)
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

}
