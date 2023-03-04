use oauth2::basic::BasicClient;

use oauth2::http::{HeaderMap, HeaderValue};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, TokenResponse,
    TokenUrl,
};
use reqwest::Method;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
    // token URL.
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
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new("https://localhost".to_string())?);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        .url();

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    let Some(code) = std::io::stdin().lines().next() else { return Ok(()) };
    let code = oauth2::AuthorizationCode::new(code.unwrap());

    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(pkce_verifier)
        .request(oauth2::reqwest::http_client);

    println!("Returned the following response:\n{:?}\n", token_response);

    if let Ok(token) = token_response {
        let token_string = token.access_token().secret();

        println!("Token: {}", token_string);

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

        println!("Body: {:?}", std::str::from_utf8(&response.body).unwrap());
    }

    Ok(())
}
