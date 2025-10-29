use chrono::Local;
use dotenv::dotenv;
use serenity::builder::{CreateMessage, CreatePoll, CreatePollAnswer};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use cron_poll_discord::{
    commands,
    discord::{find_guild_channel, list_guilds},
    poll::{
        cron_filter,
        domain::{Poll, PollInstance, PollInstanceAnswer},
        poll_instance_use_cases::PollUseCases
    },
    migrations::init_db
};
use poise::serenity_prelude as serenity;
use cron_poll_discord::commands::types::{Data, Error};

fn to_createpollanswers(answers: &Vec<String>) -> Vec<CreatePollAnswer> {
    let mut poll_answers: Vec<CreatePollAnswer> = vec![];
    for a in answers {
        poll_answers.push(CreatePollAnswer::new().text(a.clone()));
    }

    return poll_answers;
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::CacheReady { guilds } => {
            println!("Sender started");

            let guilds = list_guilds(ctx.clone(), guilds.clone());
            let ctx = Arc::new(ctx.clone());
            let pool = Arc::new(data.pool.clone());

            if !data.is_running.load(Ordering::Relaxed) {
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

                        let mut flatten_polls: Vec<Poll> = polls_to_lookup.into_iter().flatten().collect();
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

                        poll_use_cases.save_poll(p.clone().sent(true)).await.unwrap();

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

                data.is_running.swap(true, Ordering::Relaxed);
            }
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database = env::var("DATABASE_URL").expect("Expected DATABASE in the environment");
    let pool = init_db(&database).await.unwrap();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![commands::utilities::help(), commands::utilities::ping()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(commands::types::on_error(error)),
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    is_running: AtomicBool::new(false),
                    pool: pool.clone(),
                })
            })
        })
        .options(options)
        .build();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MESSAGE_POLLS;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
