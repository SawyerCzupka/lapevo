/// Application configuration.
pub struct Config {
    pub server_url: String,
}

impl Config {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
        }
    }
}
