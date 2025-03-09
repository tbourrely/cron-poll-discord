use std::env;
use serenity::prelude::*;
use dotenv::dotenv;
use cron_poll_discord::migrations::init_db;
use cron_poll_discord::poll::repository::{PollInstanceRepository, PollRepository};
use rusqlite::Connection;
use serenity::async_trait;
use serenity::model::event::{MessagePollVoteAddEvent, MessagePollVoteRemoveEvent};
use serenity::prelude::{Context, EventHandler};

pub struct Handler {
    pub db_name: String,
}

fn create_instance_repository(dbname: String) -> PollInstanceRepository {
    PollInstanceRepository {
        conn: Connection::open(dbname.clone()).unwrap(),
        poll_repository: PollRepository { conn: Connection::open(dbname.clone()).unwrap() },
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn poll_vote_remove(&self, _: Context, msg: MessagePollVoteRemoveEvent) {
        println!("Message id {:?}", msg.message_id);
        println!("Answer id {:?}", msg.answer_id);

        let repo = create_instance_repository(self.db_name.clone());
        let mut poll = match repo.find(msg.message_id.get()) {
            Ok(poll) => poll,
            Err(error) => panic!("Could not load poll {:?}", error),
        };

        println!("found poll: {:?}", poll);

        poll.remove_vote(msg.answer_id.get()).unwrap();
        repo.save(poll).unwrap();
    }

    async fn poll_vote_add(&self, _: Context, msg: MessagePollVoteAddEvent) {
        println!("Message id {:?}", msg.message_id);
        println!("Answer id {:?}", msg.answer_id);

        let repo = create_instance_repository(self.db_name.clone());
        let mut poll = match repo.find(msg.message_id.get()) {
            Ok(poll) => poll,
            Err(error) => panic!("Could not load poll {:?}", error),
        };

        println!("found poll: {:?}", poll);

        poll.add_vote(msg.answer_id.get()).unwrap();
        repo.save(poll).unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database = env::var("DATABASE").expect("Expected DATABASE in the environment");
    init_db(&database).ok();

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
            .event_handler(Handler{db_name: database}).await
            .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
