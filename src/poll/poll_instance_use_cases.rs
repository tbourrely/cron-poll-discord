use crate::poll::domain::{Poll, PollInstance, PollInstanceAnswer};
use crate::poll::repository::{PollInstanceRepository, PollRepository};
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

pub struct PollUseCases<'a> {
    poll_repository: PollRepository<'a>,
    poll_instance_repository: PollInstanceRepository<'a>,
}
impl PollUseCases<'_> {
    pub fn new(pool: &'_ PgPool) -> PollUseCases<'_> {
        PollUseCases {
            poll_repository: PollRepository { pool },
            poll_instance_repository: PollInstanceRepository { pool },
        }
    }

    pub async fn get_poll_by_id(&self, id: Uuid) -> Result<Poll, Box<dyn Error>> {
        let poll = self.poll_repository.find_by_id(id).await?;
        Ok(poll)
    }

    pub async fn get_polls(&self) -> Result<Vec<Poll>, Box<dyn Error>> {
        let polls = self.poll_repository.get_all().await?;
        Ok(polls)
    }

    pub async fn save_poll(&self, poll: Poll) -> Result<Uuid, Box<dyn Error>> {
        self.poll_repository.save(&poll).await
    }

    pub async fn delete_poll_by_id(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        self.poll_repository.delete_poll(id).await?;
        Ok(())
    }

    pub async fn get_poll_instances_by_poll_id(
        &self,
        id: Uuid,
    ) -> Result<Vec<PollInstance>, Box<dyn Error>> {
        let poll = self.poll_repository.find_by_id(id).await?;
        let poll_instance = self.poll_instance_repository.find_by_poll(poll).await?;
        Ok(poll_instance)
    }

    pub async fn get_poll_instance_by_id(&self, id: i64) -> Result<PollInstance, Box<dyn Error>> {
        let mut instance = self.poll_instance_repository.find(id).await?;
        let poll = self
            .poll_repository
            .find_by_id(instance.poll_uuid.unwrap())
            .await?;
        let answers = self.get_answers_by_instance_id(id).await?;

        instance.answers = answers;
        instance.poll = Some(poll);

        Ok(instance)
    }

    pub async fn get_answers_by_instance_id(
        &self,
        id: i64,
    ) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let answers = self.poll_instance_repository.find_answers(id).await?;
        Ok(answers)
    }

    pub async fn get_poll_instance_answers_from_poll_id(
        &self,
        poll_id: Uuid,
    ) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let poll_instance_answers = self
            .poll_instance_repository
            .find_answers_by_poll_id(poll_id)
            .await?;
        Ok(poll_instance_answers)
    }

    pub async fn save_instance(&self, instance: PollInstance) -> Result<(), Box<dyn Error>> {
        self.poll_instance_repository.save(instance).await?;
        Ok(())
    }
}
