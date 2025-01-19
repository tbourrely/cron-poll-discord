use cron_poll_discord::poll::{parser, cron_filter};
use std::env;
use std::time::Duration;
use chrono::Local;
use serenity::prelude::*;
use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreatePoll, CreatePollAnswer};
use cron_poll_discord::poll::domain::{Poll, PollAnswerCount};
use rusqlite::Connection;
use cron_poll_discord::poll::repository::PollRepository;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Handler {
    is_running: AtomicBool,
    database: String
}

#[async_trait]
impl EventHandler for Handler {

    async fn cache_ready(&self, ctx: Context, ids: Vec<serenity::all::GuildId>) {
        println!("Daemon started");


        if ids.len() > 1 {
            println!("Too much guilds, can't process");
            return;
        }

        let ctx_clone = ctx.clone();
        let user_id = ctx_clone.cache.clone().current_user().id;
        let guild_ref = ctx_clone.cache.clone().guild(ids[0]).unwrap().clone();
        let channel_ref = guild_ref.clone().default_channel(user_id).unwrap().clone();
        let channel = Arc::new(channel_ref);
        let ctx = Arc::new(ctx_clone);
        let database = Arc::new(self.database.clone());

        if !self.is_running.load(Ordering::Relaxed) {

            tokio::spawn(async move {
                loop {
                    let now = Local::now();
                    let polls = cron_filter::filter(parser::parse(), &now);
                    println!("now : {:?}", now);
                    println!("number of polls to send : {:?}", polls.len());

                    for p in polls {
                        println!("{:?}", p);

                        let mut answers: Vec<String> = vec![];
                        for a in p.answers {
                            answers.push(a.answer);
                        }
                        let mut poll_answers: Vec<CreatePollAnswer> = vec![];
                        for answer in answers {
                            poll_answers.push(CreatePollAnswer::new().text(answer));
                        }

                        let poll = CreatePoll::new()
                            .question(p.question)
                            .answers(poll_answers)
                            .duration(std::time::Duration::from_secs(60 * 60 * 24 * 7));

                        let poll_msg = CreateMessage::new().poll(poll);
                        let sent_details = channel.send_message(&ctx, poll_msg).await.unwrap();

                        let sent_poll_details = sent_details.poll.unwrap();
                        let question = sent_poll_details.question.text.unwrap();

                        let mut answers: Vec<PollAnswerCount> = vec![];
                        for answer in sent_poll_details.answers {
                            answers.push(PollAnswerCount {
                                id: answer.answer_id.get(),
                                answer: answer.poll_media.text.unwrap(),
                                votes: 0,
                            })
                        }

                        let poll_to_save = Poll {
                            id: sent_details.id.get(),
                            cron: String::new(),
                            question,
                            answers,
                        };

                        let conn = Connection::open(database.to_string()).unwrap();
                        let repo = PollRepository { conn };

                        repo.save(poll_to_save).unwrap();
                    }

                    let _ = tokio::time::sleep(Duration::from_secs(1)).await;
                }

            });

            self.is_running.swap(true, Ordering::Relaxed);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database = env::var("DATABASE").expect("Expected DATABASE in the environment");

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MEMBERS
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILD_MESSAGE_POLLS;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
    Client::builder(&token, intents).event_handler(Handler{is_running: AtomicBool::new(false), database}).await.expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
