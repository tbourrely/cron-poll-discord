use chrono::Local;
use cron_poll_discord::discord::{find_guild_channel, list_guilds};
use cron_poll_discord::poll::cron_filter;
use cron_poll_discord::poll::domain::{Poll, PollInstance, PollInstanceAnswer};
use dotenv::dotenv;
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

                    let found_groups = poll_use_cases.get_poll_groups().await.unwrap();

                    let mut polls_to_lookup: Vec<Vec<Poll>> = Vec::new();
                    for group in found_groups {
                        let now = Local::now();
                        let polls = cron_filter::filter(group.polls, &now);

                        println!("now : {:?}", now);
                        println!("number of polls to send : {:?}", polls.len());

                        polls_to_lookup.push(polls)
                    }

                    let mut flatten_polls: Vec<Poll> =
                        polls_to_lookup.into_iter().flatten().collect();
                    flatten_polls.retain(|x| x.sent == false);

                    for p in flatten_polls {
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

                        let poll_answers = to_createpollanswers(&p.answers);

                        let mut create_poll = CreatePoll::new()
                            .question(p.question.clone())
                            .answers(poll_answers)
                            .duration(Duration::from_secs((p.duration as u64).into()));

                        if p.multiselect {
                            create_poll = create_poll.allow_multiselect();
                        }

                        let poll_msg = CreateMessage::new().poll(create_poll);
                        let sent_details = channel.send_message(&ctx, poll_msg).await.unwrap();
                        let sent_poll_details = sent_details.poll.unwrap();

                        let mut answers: Vec<PollInstanceAnswer> = vec![];
                        for answer in sent_poll_details.answers {
                            answers.push(PollInstanceAnswer {
                                discord_answer_id: answer.answer_id.get() as i64,
                                answer: answer.poll_media.text.unwrap(),
                                votes: 0,
                            })
                        }

                        poll_use_cases
                            .save_poll(p.clone().sent(true))
                            .await
                            .unwrap();

                        let instance = PollInstance {
                            id: sent_details.id.get() as i64,
                            sent_at: sent_details.timestamp.unix_timestamp(),
                            answers,
                            poll_uuid: None,
                            poll: Some(p),
                        };

                        poll_use_cases.save_instance(instance).await.unwrap();
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
