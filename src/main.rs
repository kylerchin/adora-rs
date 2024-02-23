mod translation;

use poise::serenity_prelude::{self as serenity};
use serde::Deserializer;
use serde_aux::prelude::{deserialize_number_from_string, deserialize_option_number_from_string};
use translation::tr;
mod genius;
use chrono::prelude::{DateTime, Utc};
use genius::genius_lyrics;
mod data_types;
use std::time::{Duration, SystemTime};

use data_types::{Context, Data, Error};

use std::collections::HashMap;

use serde::Deserialize;

#[poise::command(slash_command)]
async fn age(ctx: Context<'_>, user: Option<serenity::User>) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ping_info = ctx.ping().await.as_micros() as f64 / 1000.;
    let response = format!("{}ms", ping_info);
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn lyrics(ctx: Context<'_>, search: String) -> Result<(), Error> {
    genius_lyrics(ctx, search).await
}

fn valid_yt_url(input: &url::Url) -> bool {
    let host = input.host_str();

    if host.is_some() {
        let host = host.unwrap();

        if host == "youtube.com"
            || host == "www.youtube.com"
            || host == "music.youtube.com"
            || host == "youtu.be"
        {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

fn get_video_id_from_video_url(input: &url::Url) -> Option<String> {
    let host = input.host_str().unwrap();

    if host == "youtube.com" || host == "www.youtube.com" || host == "music.youtube.com" {
        let hash_query: HashMap<_, _> = input.query_pairs().into_owned().collect();

        let id = hash_query.get("v");

        return match id {
            Some(id) => Some(id.clone()),
            None => None,
        };
    }

    if host == "youtu.be" {
        let path = input.path().clone().replace("/", "");

        return Some(path);
    }

    None
}

#[derive(Clone, Deserialize)]
struct YouTubeResponseItem {
    kind: String,
    etag: String,
    id: String,
    snippet: YouTubeResponseSnippet,
    #[serde(rename = "statistics")]
    statistics: YouTubeResponseStatistics,
}

#[derive(Clone, Deserialize)]
struct YouTubeThumbnailList {
    default: Option<ThumbnailItem>,
    medium: Option<ThumbnailItem>,
    high: Option<ThumbnailItem>,
    standard: Option<ThumbnailItem>,
    maxres: Option<ThumbnailItem>,
}

fn thumbnail_option_to_empty_url(x: Option<&ThumbnailItem>) -> String {
    match x {
        Some(x) => x.url.clone(),
        None => String::from(""),
    }
}

#[derive(Clone, Deserialize)]
struct ThumbnailItem {
    url: String,
    width: u16,
    height: u16,
}

#[derive(Clone, Deserialize)]
struct YouTubeResponseSnippet {
    #[serde(rename = "publishedAt")]
    published_at: String,
    #[serde(rename = "channelId")]
    channel_id: String,
    title: String,
    thumbnails: YouTubeThumbnailList,
    tags: Vec<String>,
}

fn deserialize_u64_option<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = Option::<String>::deserialize(deserializer)?;
    match string {
        Some(string) => {
            match string.parse::<u64>() {
                Ok(value) => Ok(Some(value)),
                Err(_) => Ok(None), // Handle non-numeric strings as None
            }
        }
        _ => Ok(None),
    }
}

#[derive(Clone, Deserialize)]
struct YouTubeResponseStatistics {
    #[serde(
        rename = "viewCount",
        deserialize_with = "deserialize_number_from_string"
    )]
    view_count: u64,
    #[serde(
        rename = "commentCount",
        deserialize_with = "deserialize_number_from_string"
    )]
    comment_count: u64,
    #[serde(default)]
    #[serde(rename = "likeCount", deserialize_with = "deserialize_u64_option")]
    like_count: Option<u64>,
}

#[derive(Clone, Deserialize)]
struct YouTubeResponse {
    kind: String,
    etag: String,
    items: Vec<YouTubeResponseItem>,
}

fn iso8601(st: &SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
    // formats like "2001-07-08T00:34:60.026490+09:30"
}

fn iso8601_now() -> String {
    let now = SystemTime::now();
    iso8601(&now)
}

async fn send_yt_chart(video_id: String, ctx: Context<'_>) {
    let api_key = std::env::var("YOUTUBE_API").expect("missing YOUTUBE_API");

    let path_yt = format!("https://youtube.googleapis.com/youtube/v3/videos?part=snippet,statistics,status,liveStreamingDetails&id={video_id}&key={api_key}");

    let fetch = reqwest::get(path_yt).await;

    match fetch {
        Ok(fetch) => {
            let text = fetch.text().await.unwrap();

            let parse_youtube = serde_json::from_str::<YouTubeResponse>(text.as_str());

            match parse_youtube {
                Ok(parse_youtube) => {
                    if parse_youtube.items.len() > 0 {
                        let item = &parse_youtube.items[0];

                        let iso_fmt = iso8601_now();

                        let response = format!(
                            "***{}***\n{}\nViews: {}\nComments: {}\nLikes: {:?}\n{}",
                            item.snippet.title,
                            iso_fmt,
                            item.statistics.view_count,
                            item.statistics.comment_count,
                            item.statistics.like_count,
                            thumbnail_option_to_empty_url(item.snippet.thumbnails.maxres.as_ref())
                        );
                        ctx.say(response).await;
                    } else {
                        let response = format!("No data found!");
                        ctx.say(response).await;
                    }
                }
                Err(err) => {
                    println!("{:#?}", err);

                    let response =
                        format!("Fetched but failed to deserialise response from Google servers.");
                    ctx.say(response).await;
                }
            }
        }
        Err(fetch) => {
            let response = format!("Valid url but failed to fetch.");
            ctx.say(response).await;
        }
    }
}

#[poise::command(slash_command)]
async fn yt(ctx: Context<'_>, search: String) -> Result<(), Error> {
    //let response = format!("You searched for {}.\n This function is being rewritten in Rust over the next few days. Check https://discord.gg/aYdFXm6JPe for more updates.",search);
    //ctx.say(response).await?;
    match search.trim().chars().count() {
        0 => {
            let response = format!("The correct format for Youtube Video Searches is `a!youtube [youtube url / search string]`");
            ctx.say(response).await?;
        }
        _ => {
            let url_search = url::Url::parse(search.as_str());

            match url_search {
                Ok(url_search) => {
                    if valid_yt_url(&url_search) {
                        let video_id = get_video_id_from_video_url(&url_search);

                        match video_id {
                            Some(video_id) => {
                                //lookup
                                send_yt_chart(video_id, ctx).await;
                            }
                            None => {
                                let response =
                                    format!("Could not get a video id from url {}", search);
                                ctx.say(response).await?;
                            }
                        }
                    } else {
                        let response = format!(
                            "You searched for {}.\n This is not a valid youtube url!",
                            search
                        );
                        ctx.say(response).await?;
                    }
                }
                Err(_) => {
                    let response = format!("You searched for {}.\n This function currently does not support word searches. Try again with a URL! Check https://discord.gg/aYdFXm6JPe for more updates.",search);
                    ctx.say(response).await?;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut commands = vec![age(), ping(), lyrics(), yt()];
    let translations = translation::read_ftl().expect("failed to read translation files");
    translation::apply_translations(&translations, &mut commands);

    //shadow commands so it can't be changed anymore
    let commands = commands;

    let _ = std::env::var("YOUTUBE_API").expect("missing YOUTUBE_API");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands,
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    translations,
                    reqwestclient: reqwest::Client::new(),
                })
            })
        });

    framework.run().await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn languages() {
        let mut commands = vec![age(), ping(), lyrics()];
        let translations = translation::read_ftl().expect("failed to read translation files");
        translation::apply_translations(&translations, &mut commands);

        //shadow commands so it can't be changed anymore
        let commands = commands;
    }
}
