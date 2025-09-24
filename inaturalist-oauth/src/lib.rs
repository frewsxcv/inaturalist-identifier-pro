use oauth2::basic::BasicClient;
use oauth2::http::{HeaderMap, HeaderValue, Method};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Default)]
struct MyConfig {
    api_token: Option<String>,
}


pub fn get_api_token() -> Result<String, Box<dyn std::error::Error>> {
    let mut cfg: MyConfig = confy::load("inaturalist-fetch", None)?;

    if let Some(token) = &cfg.api_token {
        println!("Using existing API token: {}", token);
        return Ok(token.clone());
    }

    let client = BasicClient::new(
        ClientId::new("h_gk-W1QMcTwTAH4pmo3TEitkJzeeZphpsj7TM_yq18".to_string()),
        Some(ClientSecret::new(
            "RLRDkivCGzGMGqWrV4WHIA7NJ7CqL0nhQ5n9lbIipCw".to_string(),
        )),
        AuthUrl::new("https://www.inaturalist.org/oauth/authorize".to_string())?,
        Some(TokenUrl::new(
            "https://www.inaturalist.org/oauth/token".to_string(),
        )?),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080".to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Opening browser to: {}", auth_url);
    opener::open(auth_url.to_string())?;

    let listener = TcpListener::bind("127.0.0.1:8080")?;
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

    println!("OAuth access token: {}", token_string);

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
    println!("OAuth API token: {}", response.api_token);
    cfg.api_token = Some(response.api_token.clone());
    confy::store("inaturalist-fetch", None, cfg)?;
    Ok(response.api_token)
}

#[derive(Deserialize)]
struct ApiTokenResponse {
    api_token: String,
}
