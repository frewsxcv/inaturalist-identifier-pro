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
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

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
    pub fn get_api_token(self) -> Result<String, Box<dyn std::error::Error>> {
        let redirect_url = format!("http://localhost:{}", self.port);
        let client = BasicClient::new(
            self.client_id,
            Some(self.client_secret),
            AuthUrl::new("https://www.inaturalist.org/oauth/authorize".to_string())?,
            Some(TokenUrl::new(
                "https://www.inaturalist.org/oauth/token".to_string(),
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url)?);

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, _csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();

        log::info!("Opening browser to: {auth_url}");
        opener::open(auth_url.to_string())?;

        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;
        let mut code: Option<AuthorizationCode> = None;
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                {
                    let mut reader = BufReader::new(&stream);
                    let mut request_line = String::new();
                    reader.read_line(&mut request_line).unwrap();

                    let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                    let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                    let code_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "code"
                        })
                        .unwrap();

                    let (_, value) = code_pair;
                    code = Some(AuthorizationCode::new(value.into_owned()));
                }

                let message = "<h1>Success!</h1><p>You can close this window now.</p>";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes()).unwrap();
                break;
            }
        }

        let token_response = client
            .exchange_code(code.unwrap())
            .set_pkce_verifier(pkce_verifier)
            .request(oauth2::reqwest::http_client)
            .unwrap();

        let token_string = token_response.access_token().secret();

        log::info!("OAuth access token: {token_string}");

        let mut headers = HeaderMap::new();
        headers.append(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {token_string}")).unwrap(),
        );
        let response = oauth2::reqwest::http_client(oauth2::HttpRequest {
            body: vec![],
            headers,
            url: "https://www.inaturalist.org/users/api_token"
                .try_into()
                .unwrap(),
            method: Method::GET,
        })
        .unwrap();

        let response: ApiTokenResponse = serde_json::from_slice(&response.body).unwrap();
        log::info!("OAuth API token: {}", response.api_token);
        Ok(response.api_token)
    }
}

#[derive(Deserialize)]
struct ApiTokenResponse {
    api_token: String,
}
