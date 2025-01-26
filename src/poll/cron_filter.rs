use crate::poll::domain::Poll;
use croner::Cron;
use chrono::{DateTime, TimeZone};

pub fn filter<Tz: TimeZone>(polls: Vec<Poll>, datetime: &DateTime<Tz>) -> Vec<Poll> {
    let mut filtered: Vec<Poll> = vec![];
    let datetime_without_ns = DateTime::parse_from_rfc3339(&datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)).unwrap();

    for p in polls {
        let cron = Cron::new(&p.cron).with_seconds_optional().parse().unwrap();
        let matching = cron.is_time_matching(&datetime_without_ns).unwrap();
        if matching {
            filtered.push(p);
        }
    }

    return filtered;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_filter() {
        let polls: Vec<Poll> = vec![
            Poll{id: 0, cron: String::from("* * * * *"), question: String::new(), answers: vec![]}
        ];

        let date_str = "2020-04-12T22:10:00+02:00";
        let datetime = DateTime::parse_from_rfc3339(date_str).unwrap();
        let result = filter(polls, &datetime);
        assert_eq!(1, result.len());
    }

    #[test]
    fn test_filter_strip_ns() {
        let polls: Vec<Poll> = vec![
            Poll{id: 0, cron: String::from("* * * * *"), question: String::new(), answers: vec![]}
        ];

        let date_str = "2025-01-25T23:18:00.383550841+01:00";
        let datetime = DateTime::parse_from_rfc3339(date_str).unwrap();
        let result = filter(polls, &datetime);
        assert_eq!(1, result.len());
    }

    #[test]
    fn test_filter_no_match() {
        let polls: Vec<Poll> = vec![
            Poll{id: 0, cron: String::from("* * * * *"), question: String::new(), answers: vec![]}
        ];

        let date_str = "2020-04-12T22:10:01+02:00";
        let datetime = DateTime::parse_from_rfc3339(date_str).unwrap();
        let result = filter(polls, &datetime);
        assert_eq!(0, result.len());
    }
}
