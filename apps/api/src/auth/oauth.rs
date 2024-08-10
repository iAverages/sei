use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

const TOKEN_URL: &str = "https://myanimelist.net/v1/oauth2/token";
const AUTH_URL: &str = "https://myanimelist.net/v1/oauth2/authorize";

pub fn create_oauth_client(
    api_url: String,
    client_id: String,
    client_secret: String,
) -> BasicClient {
    let redirect_url = api_url + "/oauth/mal/callback";
    let auth_url = AuthUrl::new(AUTH_URL.to_string()).expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(TOKEN_URL.to_string()).expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("Invalid redirect URL"))
}
