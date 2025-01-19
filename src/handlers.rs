use crate::poll::repository::PollRepository;

use rusqlite::Connection;
use serenity::async_trait;
use serenity::model::event::{MessagePollVoteAddEvent, MessagePollVoteRemoveEvent};
use serenity::prelude::{Context, EventHandler};

pub struct Handler {
    pub db_name: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn poll_vote_remove(&self, _: Context, msg: MessagePollVoteRemoveEvent) {
        println!("Message id {:?}", msg.message_id);
        println!("Answer id {:?}", msg.answer_id);

        let conn = Connection::open(self.db_name.as_str()).unwrap();
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

        let conn = Connection::open(self.db_name.as_str()).unwrap();
        let repo = PollRepository { conn };
        let mut poll = match repo.find(msg.message_id.get()) {
            Ok(poll) => poll,
            Err(error) => panic!("Could not load poll {:?}", error),
        };

        println!("found poll: {:?}", poll);

        poll.add_vote(msg.answer_id.get()).unwrap();
        repo.save(poll).unwrap();
    }
}
