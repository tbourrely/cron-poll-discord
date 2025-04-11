use cron_poll_discord::migrations::init_db;
use cron_poll_discord::poll::repository::{PollInstanceRepository, PollRepository};
use dotenv::dotenv;
use serenity::async_trait;
use serenity::model::event::{MessagePollVoteAddEvent, MessagePollVoteRemoveEvent};
use serenity::prelude::*;
use serenity::prelude::{Context, EventHandler};
use std::env;
use tokio::sync::mpsc;

enum Command {
    Add { poll_id: u64, answer_id: u64 },
    Remove { poll_id: u64, answer_id: u64 },
}

struct Handler {
    sender: tokio::sync::mpsc::Sender<Command>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn poll_vote_remove(&self, _: Context, msg: MessagePollVoteRemoveEvent) {
        let msg_id = msg.message_id.get();
        let answer_id = msg.answer_id.get();

        println!("[bot] removing a vote to answer {:?}", answer_id);

        self.sender
            .send(Command::Remove {
                poll_id: msg_id,
                answer_id,
            })
            .await
            .unwrap();
    }

    async fn poll_vote_add(&self, _: Context, msg: MessagePollVoteAddEvent) {
        let msg_id = msg.message_id.get();
        let answer_id = msg.answer_id.get();

        println!("[bot] adding a vote to answer {:?}", answer_id);

        self.sender
            .send(Command::Add {
                poll_id: msg_id,
                answer_id,
            })
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database = env::var("DATABASE_URL").expect("Expected DATABASE in the environment");
    let pool = init_db(&database).await.unwrap();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_POLLS;

    let (tx, mut rx) = mpsc::channel::<Command>(32);
    let manager_pool = pool.clone();
    let manager = tokio::spawn(async move {
        use Command::*;

        println!("starting manager...");
        let repo = PollInstanceRepository {
            pool: &manager_pool,
            poll_repository: &PollRepository {
                pool: &manager_pool,
            },
        };

        while let Some(cmd) = rx.recv().await {
            println!("[manager] new task received");
            match cmd {
                Add { poll_id, answer_id } => {
                    let mut poll = match repo.find(poll_id as i64).await {
                        Ok(poll) => poll,
                        Err(error) => panic!("Could not load poll {:?}", error),
                    };
                    poll.add_vote(answer_id as i64).unwrap();
                    repo.save(poll).await.unwrap();
                }
                Remove { poll_id, answer_id } => {
                    let mut poll = match repo.find(poll_id as i64).await {
                        Ok(poll) => poll,
                        Err(error) => panic!("Could not load poll {:?}", error),
                    };
                    poll.remove_vote(answer_id as i64).unwrap();
                    repo.save(poll).await.unwrap();
                }
            };
        }
    });

    let bot = tokio::spawn(async move {
        println!("starting bot...");
        let handler = Handler { sender: tx };

        // Create a new instance of the Client, logging in as a bot.
        let mut client = Client::builder(&token, intents)
            .event_handler(handler)
            .await
            .expect("Err creating client");

        // Start listening for events by starting a single shard
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });

    manager.await.unwrap();
    bot.await.unwrap();
}
