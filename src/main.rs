mod poll;
mod handlers;
mod migrations;

use std::env;
use serenity::prelude::*;
use dotenv::dotenv;
use migrations::init_db;

static DATABASE: &str = "polls.sqlite";

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_db().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_POLLS;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents)
            .event_handler(crate::handlers::Handler).await
            .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
