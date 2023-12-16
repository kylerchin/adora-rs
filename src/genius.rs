use poise::serenity_prelude as serenity;
use crate::data_types::{Context, Data, Error};

pub async fn genius_lyrics(ctx: Context<'_>, search: String) -> Result<(), Error> {

    let geniustoken = std::env::var("geniustoken").expect("Missing Genius API Token in env var `geniusapitoken`");

    let response = make_genius_req(&ctx.data().reqwestclient, search.as_str(), geniustoken.as_str()).await;

    if response.is_err() {
        return Err(Box::new(response.unwrap_err()));
    }

    let response = response.unwrap();

    Ok(())
}

async fn make_genius_req(client: &reqwest::Client, query: &str, token: &str) -> Result<reqwest::Response, reqwest::Error>
{
    return client.get(
        format!("https://api.genius.com/search?q={}",query)
    ).header("Authorization", format!("Bearer {token}"))
    .send().await;
}