use inaturalist_oauth::TokenDetails;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub token: Option<TokenDetails>,
}

impl Config {
    /// Load the configuration from the default location
    pub fn load() -> Result<Self, Box<dyn Error>> {
        Ok(confy::load("inaturalist-identifier-pro", None)?)
    }

    /// Save the configuration to the default location
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        confy::store("inaturalist-identifier-pro", None, self)?;
        Ok(())
    }

    /// Check if we have a valid (non-expired) token
    pub fn has_valid_token(&self) -> bool {
        if let Some(token) = &self.token {
            token.expires_at >= std::time::SystemTime::now()
        } else {
            false
        }
    }

    /// Get the API token if it's valid, otherwise None
    pub fn get_api_token(&self) -> Option<String> {
        if self.has_valid_token() {
            self.token.as_ref().map(|t| t.api_token.clone())
        } else {
            None
        }
    }

    /// Set a new token and save the configuration
    pub fn set_token(&mut self, token: TokenDetails) -> Result<(), Box<dyn Error>> {
        self.token = Some(token);
        self.save()?;
        Ok(())
    }

    /// Clear the token and save the configuration
    pub fn clear_token(&mut self) -> Result<(), Box<dyn Error>> {
        self.token = None;
        self.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.token.is_none());
        assert!(!config.has_valid_token());
        assert!(config.get_api_token().is_none());
    }
}
