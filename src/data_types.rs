use crate::translation::tr;
use reqwest::Client;

pub struct Data {
    // User data, which is stored and accessible in all command invocations
    pub translations: crate::translation::Translations,
    pub reqwestclient: reqwest::Client,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
