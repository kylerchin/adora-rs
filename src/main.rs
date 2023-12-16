mod translation;

use poise::serenity_prelude as serenity;
use translation::tr;

mod genius;
use genius::genius_lyrics;

mod data_types;

use data_types::{Data, Error, Context};

#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn ping(
    ctx: Context<'_>
) -> Result<(), Error> {
    let ping_info = ctx.ping().await.as_micros() as f64 / 1000.;
    let response = format!("{}ms",ping_info);
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn lyrics(
    ctx: Context<'_>,
    search: String
) -> Result<(), Error> {
    genius_lyrics(ctx, search)
}

#[tokio::main]
async fn main() {
    let mut commands = vec![age(),ping(),lyrics()];
    let translations = translation::read_ftl().expect("failed to read translation files");
    translation::apply_translations(&translations, &mut commands);

    //shadow commands so it can't be changed anymore
    let commands = commands;

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
                Ok(Data {translations})
            })
        });

    framework.run().await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn languages() {
        let mut commands = vec![age(),ping(),lyrics()];
        let translations = translation::read_ftl().expect("failed to read translation files");
        translation::apply_translations(&translations, &mut commands);

        //shadow commands so it can't be changed anymore
        let commands = commands;
    }
}