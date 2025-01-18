use crate::poll::domain::{Poll, PollAnswerCount};
use crate::poll::repository::PollRepository;

use rusqlite::Connection;
use serenity::async_trait;
use serenity::builder::{CreateMessage, CreatePoll, CreatePollAnswer};
use serenity::model::channel::Message;
use serenity::model::event::{MessagePollVoteAddEvent, MessagePollVoteRemoveEvent};
use serenity::prelude::{Context, EventHandler};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn poll_vote_remove(&self, _: Context, msg: MessagePollVoteRemoveEvent) {
        println!("Message id {:?}", msg.message_id);
        println!("Answer id {:?}", msg.answer_id);

        let conn = Connection::open(crate::DATABASE).unwrap();
        let repo = PollRepository { conn };
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

        let conn = Connection::open(crate::DATABASE).unwrap();
        let repo = PollRepository { conn };
        let mut poll = match repo.find(msg.message_id.get()) {
            Ok(poll) => poll,
            Err(error) => panic!("Could not load poll {:?}", error),
        };

        println!("found poll: {:?}", poll);

        poll.add_vote(msg.answer_id.get()).unwrap();
        repo.save(poll).unwrap();
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let conn = Connection::open(crate::DATABASE).unwrap();

        if msg.content == "!poll" {
            poll_create(&ctx, &msg, conn).await;
        }
    }
}

async fn poll_create(ctx: &Context, msg: &Message, conn: Connection) {
    println!("received poll creation request");

    let poll = CreatePoll::new()
        .question("Quel jour ?")
        .answers(vec![
            CreatePollAnswer::new().text("Lundi".to_string()),
            CreatePollAnswer::new().text("Mardi".to_string()),
            CreatePollAnswer::new().text("Mercredi".to_string()),
            CreatePollAnswer::new().text("Jeudi".to_string()),
            CreatePollAnswer::new().text("Vendredi".to_string()),
            CreatePollAnswer::new().text("Samedi".to_string()),
            CreatePollAnswer::new().text("Dimanche".to_string()),
        ])
        .duration(std::time::Duration::from_secs(60 * 60 * 24 * 7));

    let poll_msg = CreateMessage::new().poll(poll);
    let sent_msg = msg.channel_id.send_message(&ctx.http, poll_msg).await;

    let sent_details = sent_msg.unwrap();
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
        question,
        answers,
    };

    let repo = PollRepository { conn };
    match repo.save(poll_to_save) {
        Ok(_) => (),
        Err(error) => panic!("could not save poll {:?}", error),
    };
}
