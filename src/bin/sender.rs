use chrono::Local;
use cron_poll_discord::discord::{find_guild_channel, list_guilds};
use cron_poll_discord::poll::cron_filter;
use cron_poll_discord::poll::domain::{Poll as DomainPoll, PollInstance, PollInstanceAnswer};
use dotenv::dotenv;
use serenity::all::create_poll::Ready;
use serenity::all::{GuildChannel, Message};
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreatePoll, CreatePollAnswer};
use serenity::prelude::*;
use sqlx::PgPool;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cron_poll_discord::migrations::init_db;
use cron_poll_discord::poll::poll_instance_use_cases::PollUseCases;

struct Handler {
    is_running: AtomicBool,
    pool: PgPool,
}

fn to_createpollanswers(answers: &Vec<String>) -> Vec<CreatePollAnswer> {
    let mut poll_answers: Vec<CreatePollAnswer> = vec![];
    for a in answers {
        poll_answers.push(CreatePollAnswer::new().text(a.clone()));
    }

    return poll_answers;
}

// create discord polls in batches of 10 answers
fn create_discord_polls(poll: &DomainPoll) -> Vec<CreatePoll<Ready>> {
    let mut polls: Vec<CreatePoll<Ready>> = vec![];

    let poll_answers = to_createpollanswers(&poll.answers);

    let batched_answers = poll_answers.chunks(10);
    for chunk in batched_answers {
        let mut create_poll = CreatePoll::new()
            .question(poll.question.clone())
            .answers(chunk.to_vec())
            .duration(Duration::from_secs((poll.duration as u64).into()));

        if poll.multiselect {
            create_poll = create_poll.allow_multiselect();
        }

        polls.push(create_poll);
    }

    return polls;
}

async fn send_discord_polls(
    polls: Vec<CreatePoll<Ready>>,
    channel: &GuildChannel,
    ctx: &Arc<Context>,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    for p in polls {
        let poll_msg = CreateMessage::new().poll(p);
        messages.push(channel.send_message(&ctx, poll_msg).await.unwrap());
    }

    return messages;
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, ids: Vec<serenity::all::GuildId>) {
        println!("Sender started");

        let guilds = list_guilds(ctx.clone(), ids.clone());
        let ctx = Arc::new(ctx.clone());
        let pool = Arc::new(self.pool.clone());

        if !self.is_running.load(Ordering::Relaxed) {
            tokio::spawn(async move {
                loop {
                    let poll_use_cases = PollUseCases::new(&pool);
                    let now = Local::now();
                    let polls =
                        cron_filter::filter(poll_use_cases.get_polls().await.unwrap(), &now);
                    println!("number of polls to send : {:?}", polls.len());

                    for p in polls {
                        println!("{:?}", p);

                        let channels =
                            find_guild_channel(guilds.clone(), p.guild.clone(), p.channel.clone());

                        if channels.len() == 0 {
                            eprintln!(
                                "No channel found for: guild {:?} - channel {:?}",
                                p.guild.clone(),
                                p.channel.clone()
                            );
                            continue;
                        }

                        if channels.len() > 1 {
                            eprintln!(
                                "Multiple channels found for: guild {:?} - channel {:?}",
                                p.guild.clone(),
                                p.channel.clone()
                            );
                            continue;
                        }

                        let channel = channels[0].clone();

                        let polls_to_create = create_discord_polls(&p);
                        let created_polls_messages =
                            send_discord_polls(polls_to_create, &channel, &ctx).await;

                        if created_polls_messages.len() == 0 {
                            eprintln!(
                                "No poll messages created for: guild {:?} - channel {:?}",
                                p.guild.clone(),
                                p.channel.clone()
                            );
                            continue;
                        }

                        // use the timestamp of the first poll message as the sent_at timestamp for
                        // all poll instances
                        let timestamp = created_polls_messages[0].timestamp.unix_timestamp();

                        // create one poll instance per poll message created
                        for poll_message in created_polls_messages {
                            let poll = poll_message.poll.unwrap();
                            let answers = poll
                                .answers
                                .into_iter()
                                .map(|a| PollInstanceAnswer {
                                    discord_answer_id: a.answer_id.get() as i64,
                                    answer: a.poll_media.text.unwrap(),
                                    votes: 0,
                                })
                                .collect::<Vec<PollInstanceAnswer>>();

                            let instance = PollInstance {
                                id: poll_message.id.get() as i64,
                                sent_at: timestamp,
                                answers,
                                poll_uuid: None,
                                poll: Some(p.clone()),
                            };

                            poll_use_cases.save_instance(instance).await.unwrap();
                        }

                        poll_use_cases
                            .save_poll(p.clone().sent(true))
                            .await
                            .unwrap();
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
    let database = env::var("DATABASE_URL").expect("Expected DATABASE in the environment");
    let pool = init_db(&database).await.unwrap();

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
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            is_running: AtomicBool::new(false),
            pool: pool.clone(),
        })
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
