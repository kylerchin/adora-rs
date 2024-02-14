mod translation;

use poise::serenity_prelude::{self as serenity};
use translation::tr;

mod genius;
use genius::genius_lyrics;

mod data_types;

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

#[derive(Clone,Deserialize)]
struct YouTubeResponseItem {
    kind: String,
    etag: String,
    id: String,
    snippet: YouTubeResponseSnippet
}

#[derive(Clone,Deserialize)]
struct YouTubeThumbnailList {
    default: Option<ThumbnailItem>,
    medium: Option<ThumbnailItem>,
    high: Option<ThumbnailItem>,
    standard: Option<ThumbnailItem>,
    maxres: Option<ThumbnailItem>
}

#[derive(Clone,Deserialize)]
struct ThumbnailItem {
    url: String,
    width: u16,
    height: u16
}

#[derive(Clone,Deserialize)]
struct YouTubeResponseSnippet {
    #[serde(rename = "publishedAt")]
    published_at: String,
    #[serde(rename = "channelId")]
    channel_id: String,
    title: String,
    thumbnails: YouTubeThumbnailList,
    #[serde(rename = "statistics")]
    statistics: YouTubeResponseStatistics,
    tags: Vec<String>,
}

#[derive(Clone,Deserialize)]
struct YouTubeResponseStatistics {
    #[serde(rename = "viewCount")]
    view_count: u64,
    #[serde(rename = "commentCount")]
    comment_count: u64,
    #[serde(rename = "likeCount")]
    like_count: Option<u64>
}

#[derive(Clone,Deserialize)]
struct YouTubeResponse {
    kind: String,
    etag: String,
    items: Vec<YouTubeResponseItem>
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
                    
                    let response = format!("***View Count***\nViews: {}\nComments: {}\nLikes: {:?}",item.snippet.statistics.view_count,item.snippet.statistics.comment_count, item.snippet.statistics.like_count);
                    ctx.say(response).await;
                    } else {
                        let response = format!("No data found!");
                        ctx.say(response).await; 
                    }
                },
                Err(_) => {
                    let response = format!("Fetched but failed to deserialise response from Google servers.");
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
                                send_yt_chart(video_id, ctx);
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
    let mut commands = vec![age(), ping(), lyrics()];
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
