pub struct WebAnalyzerConfig {
    pub user_agent: String,
    pub output_dir: String,
    pub accept_invalid_certs: bool,
    pub timeout: std::time::Duration,
}

impl Default for WebAnalyzerConfig {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            output_dir: "output".to_string(),
            accept_invalid_certs: true,
            timeout: std::time::Duration::from_secs(30),
        }
    }
} 