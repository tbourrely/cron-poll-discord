use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::poll::domain::{Poll, PollAnswerCount};

#[derive(Serialize, Deserialize, Debug)]
struct PollDTO {
    cron: String,
    question: String,
    answers: Vec<String>,
}

fn find_yml(dir: &Path, files: &mut Vec<DirEntry>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_yml(&path, files)?;
            } else if path.extension().unwrap() == "yml"
            || path.extension().unwrap() == "yaml" {
                files.push(entry);
            }
        }
    }
    Ok(())
}

fn parse_file(entry: &DirEntry) -> Vec<Poll> {
    let content = fs::read_to_string(entry.path()).unwrap();
    let parsed: Vec<PollDTO> = serde_yml::from_str(&content).unwrap();
    let mut result: Vec<Poll> = vec![];

    for p in parsed {
        let mut answers: Vec<PollAnswerCount> = vec![];
        for a in p.answers {
            answers.push(PollAnswerCount{id: 0, answer: a, votes: 0}); // TODO: id = 0 ??
        }
        result.push(Poll{
            id: 0,
            cron: p.cron,
            question: p.question,
            answers,
        });
    }

    return result;
}

pub fn parse() -> Vec<Poll> {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let polls_path = format!("{}/polls/", root);
    let path = Path::new(&polls_path);
    let mut found_files: Vec<DirEntry> = vec![];
    let mut parsed_polls: Vec<Poll> = vec![];

    find_yml(path, &mut found_files).unwrap();
    for file in found_files {
        let mut polls = parse_file(&file);
        parsed_polls.append(&mut polls);
    }

    return parsed_polls;
}
