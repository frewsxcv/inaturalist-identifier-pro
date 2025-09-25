//! This crate provides a simple way to obtain an iNaturalist API token using the OAuth2
//! authorization flow. It handles the process of opening a web browser for user
//! authorization, running a temporary local server to catch the redirect, and
//! exchanging the authorization code for an API token.
//!
//! ## Usage
//!
//! ```no_run
//! use inaturalist_oauth::Authenticator;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client_id = "YOUR_CLIENT_ID".to_string();
//!     let client_secret = "YOUR_CLIENT_SECRET".to_string();
//!
//!     let api_token = Authenticator::new(client_id, client_secret)
//!         .with_redirect_server_port(8081)
//!         .get_api_token()?;
//!     println!("Got iNaturalist API token: {}", api_token);
//!     Ok(())
//! }
//! ```
use oauth2::basic::BasicClient;
use oauth2::http::{HeaderMap, HeaderValue, Method};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, SystemTime};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDetails {
    pub api_token: String,
    pub expires_at: SystemTime,
}

/// Handles the iNaturalist OAuth2 flow to obtain an API token.
///
/// This struct is used to configure the authenticator and initiate the OAuth2 flow.
pub struct Authenticator {
    client_id: ClientId,
    client_secret: ClientSecret,
    port: u16,
}

impl Authenticator {
    /// Creates a new `Authenticator`.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The iNaturalist application's client ID.
    /// * `client_secret` - The iNaturalist application's client secret.
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id: ClientId::new(client_id),
            client_secret: ClientSecret::new(client_secret),
            port: 8080,
        }
    }

    /// Sets the port for the local redirect server.
    ///
    /// The default port is 8080.
    pub fn with_redirect_server_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Initiates the OAuth2 flow to get an iNaturalist API token.
    ///
    /// This method opens the user's web browser to the iNaturalist authorization page.
    /// After the user authorizes the application, it completes the OAuth2 flow,
    /// obtains an access token, and then exchanges it for a long-lived API token.
    pub async fn get_api_token(self) -> Result<TokenDetails, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(("127.0.0.1", self.port))?;
        let port = listener.local_addr()?.port();
        let redirect_url = format!("http://localhost:{}", port);
        let client = self.client(&redirect_url)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();

        log::info!("Opening browser to: {auth_url}");
        opener::open(auth_url.to_string())?;

        let code = self.listen_for_code(listener)?;

        let token_response = client
            .exchange_code(code)
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await?;

        let expires_at = SystemTime::now() + Duration::from_secs(24 * 60 * 60);

        let token_string = token_response.access_token().secret();

        log::info!("OAuth access token: {token_string}");

        let response = self.fetch_api_token(token_string).await?;

        log::info!("OAuth API token: {}", response.api_token);
        Ok(TokenDetails {
            api_token: response.api_token,
            expires_at,
        })
    }

    fn listen_for_code(
        &self,
        listener: TcpListener,
    ) -> Result<AuthorizationCode, Box<dyn std::error::Error>> {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    stream.read(&mut buffer)?;

                    let mut headers = [httparse::EMPTY_HEADER; 16];
                    let mut req = httparse::Request::new(&mut headers);
                    req.parse(&buffer)?;

                    let path = match req.path {
                        Some(path) => path,
                        None => {
                            log::error!("Malformed request: no path");
                            continue;
                        }
                    };

                    let url = Url::parse(&format!("http://localhost{path}"))?;

                    if let Some(code_pair) = url.query_pairs().find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    }) {
                        let (_, value) = code_pair;
                        let code = AuthorizationCode::new(value.into_owned());

                        let message = "<h1>Success!</h1><p>You can close this window now.</p>";
                        let response = format!(
                            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                            message.len(),
                            message
                        );
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            log::error!("Failed to write response: {e}");
                        }
                        return Ok(code);
                    } else {
                        log::error!("URL did not contain 'code' parameter: {url}");
                        let message =
                            "<h1>Error!</h1><p>Could not get authorization code. Please try again.</p>";
                        let response = format!(
                            "HTTP/1.1 400 Bad Request\r\ncontent-length: {}\r\n\r\n{}",
                            message.len(),
                            message
                        );
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            log::error!("Failed to write error response: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to accept connection: {e}");
                }
            }
        }
        Err("Server closed before receiving authorization code".into())
    }

    async fn fetch_api_token(
        &self,
        token_string: &str,
    ) -> Result<ApiTokenResponse, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.append(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {token_string}"))?,
        );
        let response = oauth2::reqwest::async_http_client(oauth2::HttpRequest {
            body: vec![],
            headers,
            url: "https://www.inaturalist.org/users/api_token".try_into()?,
            method: Method::GET,
        })
        .await?;

        Ok(serde_json::from_slice(&response.body)?)
    }

    fn client(&self, redirect_url: &str) -> Result<BasicClient, Box<dyn std::error::Error>> {
        Ok(BasicClient::new(
            self.client_id.clone(),
            Some(self.client_secret.clone()),
            AuthUrl::new("https://www.inaturalist.org/oauth/authorize".to_string())?,
            Some(TokenUrl::new(
                "https://www.inaturalist.org/oauth/token".to_string(),
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url.to_string())?))
    }
}

#[derive(Deserialize)]
struct ApiTokenResponse {
    api_token: String,
}
