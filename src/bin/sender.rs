use cron_poll_discord::poll::cron_filter;
use std::env;
use std::time::Duration;
use chrono::Local;
use serenity::prelude::*;
use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreatePoll, CreatePollAnswer};
use cron_poll_discord::poll::domain::{PollInstanceAnswer, PollInstance};
use cron_poll_discord::poll::repository::{PollRepository, PollInstanceRepository};
use cron_poll_discord::discord::{list_guilds, find_guild_channel};
use rusqlite::Connection;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct Handler {
    is_running: AtomicBool,
    database: String
}

fn to_createpollanswers(answers: &Vec<String>) -> Vec<CreatePollAnswer> {
    let mut poll_answers: Vec<CreatePollAnswer> = vec![];
    for a in answers {
        poll_answers.push(CreatePollAnswer::new().text(a.clone()));
    }

    return poll_answers;
}

#[async_trait]
impl EventHandler for Handler {


    async fn cache_ready(&self, ctx: Context, ids: Vec<serenity::all::GuildId>) {
        println!("Sender started");

        let guilds = list_guilds(ctx.clone(), ids.clone());
        let ctx = Arc::new(ctx.clone());
        let database = Arc::new(self.database.clone());

        if !self.is_running.load(Ordering::Relaxed) {

            tokio::spawn(async move {
                loop {
                    let poll_repository = PollRepository { conn: Connection::open(database.to_string()).unwrap() };
                    let poll_instance_repository = PollInstanceRepository { conn: Connection::open(database.to_string()).unwrap(), poll_repository: PollRepository { conn: Connection::open(database.to_string()).unwrap() } };

                    let found_polls = poll_repository.get_all().unwrap();

                    let now = Local::now();
                    let polls = cron_filter::filter(found_polls, &now);
                    println!("now : {:?}", now);
                    println!("number of polls to send : {:?}", polls.len());

                    for p in polls {
                        println!("{:?}", p);

                        let channels = find_guild_channel(guilds.clone(), p.guild.clone(), p.channel.clone());

                        if channels.len() == 0 {
                            eprintln!("No channel found for: guild {:?} - channel {:?}", p.guild.clone(), p.channel.clone());
                            continue;
                        }

                        if channels.len() > 1 {
                            eprintln!("Multiple channels found for: guild {:?} - channel {:?}", p.guild.clone(), p.channel.clone());
                            continue;
                        }

                        let channel = channels[0].clone();

                        let poll_answers = to_createpollanswers(&p.answers);

                        let mut create_poll = CreatePoll::new()
                            .question(p.question.clone())
                            .answers(poll_answers)
                            .duration(Duration::from_secs(p.duration.into()));
                        
                        if p.multiselect {
                            create_poll = create_poll.allow_multiselect();
                        }
                    
                        let poll_msg = CreateMessage::new().poll(create_poll);
                        let sent_details = channel.send_message(&ctx, poll_msg).await.unwrap();
                        let sent_poll_details = sent_details.poll.unwrap();

                        let mut answers: Vec<PollInstanceAnswer> = vec![];
                        for answer in sent_poll_details.answers {
                            answers.push(PollInstanceAnswer {
                                discord_answer_id: answer.answer_id.get(),
                                answer: answer.poll_media.text.unwrap(),
                                votes: 0,
                            })
                        }

                        let instance = PollInstance{
                            id: sent_details.id.get(),
                            sent_at: sent_details.timestamp.unix_timestamp(),
                            answers,
                            poll: p
                        };

                        poll_instance_repository.save(instance).unwrap();
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
