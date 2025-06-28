use std::error::Error;
use sqlx::PgPool;
use uuid::Uuid;
use crate::poll::domain::{Poll, PollGroup, PollInstance, PollInstanceAnswer};
use crate::poll::repository::{PollInstanceRepository, PollRepository};

pub struct PollUseCases<'a> {
    poll_repository: PollRepository<'a>,
    poll_instance_repository: PollInstanceRepository<'a>
}
impl PollUseCases<'_> {
    pub fn new(pool: &PgPool) -> PollUseCases {
        PollUseCases {
            poll_repository: PollRepository { pool },
            poll_instance_repository: PollInstanceRepository { pool }
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

    pub async fn get_poll_groups(&self) -> Result<Vec<PollGroup>, Box<dyn Error>> {
        let groups = self.poll_repository.get_all_poll_groups().await?;
        Ok(groups)
    }

    pub async fn save_poll(&self, poll: Poll) ->  Result<(), Box<dyn Error>> {
        self.poll_repository.save(poll).await?;
        Ok(())
    }

    pub async fn create_poll_group(&self, poll: Poll) ->  Result<Uuid, Box<dyn Error>> {
        let mut poll_group = PollGroup::new(None);

        for answer_chunk in poll.answers.chunks(10) {
            poll_group.polls.push(
                poll.clone()
                    .id(Uuid::new_v4())
                    .answers(answer_chunk.to_vec())
                    .poll_group_id(Some(poll_group.id))
            );
        }

        self.poll_repository.create_poll_group(&poll_group).await?;
        for poll in poll_group.polls {
            self.poll_repository.save(poll).await?;
        }

        Ok(poll_group.id.clone())
    }

    pub async fn update_poll_group(&self, group: PollGroup) ->  Result<(), Box<dyn Error>> {
        let poll_group_exist =  self.poll_repository.poll_group_exists(group.id).await?;

        if !poll_group_exist {
           return Err(format!("Group {} does not exist", group.id).into());
        }

        self.poll_repository.update_poll_group(&group).await?;

        for poll in group.polls {
            self.poll_repository.save(poll).await?;
        }
        Ok(())
    }

    pub async fn delete_poll_by_id(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        self.poll_repository.delete_poll(id).await?;
        Ok(())
    }

    pub async fn get_poll_instances_by_poll_id(&self, id: Uuid) -> Result<Vec<PollInstance>, Box<dyn Error>> {
        let poll = self.poll_repository.find_by_id(id).await?;
        let poll_instance = self.poll_instance_repository.find_by_poll(poll).await?;
        Ok(poll_instance)
    }

    pub async fn get_poll_instance_by_id(&self, id: i64) -> Result<PollInstance, Box<dyn Error>> {
        let mut instance = self.poll_instance_repository.find(id).await?;
        let poll = self.poll_repository.find_by_id(instance.poll_uuid.unwrap()).await?;
        let answers = self.get_answers_by_instance_id(id).await?;

        instance.answers = answers;
        instance.poll = Some(poll);

        Ok(instance)
    }

    pub async fn get_answers_by_instance_id(&self, id: i64) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let answers = self.poll_instance_repository.find_answers(id).await?;
        Ok(answers)
    }

    pub async fn get_poll_instance_answers_from_poll_id(&self, poll_id: Uuid) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let poll_instance_answers = self.poll_instance_repository.find_answers_by_poll_id(poll_id).await?;
        Ok(poll_instance_answers)
    }

    pub async fn get_poll_instance_answers_from_poll_group_id(&self, group_id: Uuid) -> Result<Vec<PollInstanceAnswer>, Box<dyn Error>> {
        let polls = self.poll_repository.find_polls_by_poll_group_id(group_id).await?;
        let poll_ids: Vec<Uuid> = polls.iter().map(|p| p.id).collect();
        let mut poll_group_poll_instance_answers: Vec<Vec<PollInstanceAnswer>> = Vec::new();

        for id in poll_ids {
            poll_group_poll_instance_answers.push(self.poll_instance_repository.find_answers_by_poll_id(id).await?);
        }

        Ok(poll_group_poll_instance_answers.into_iter().flatten().collect())
    }

    pub async fn save_instance(&self, instance: PollInstance) -> Result<(), Box<dyn Error>> {
        self.poll_instance_repository.save(instance).await?;
        Ok(())
    }
}