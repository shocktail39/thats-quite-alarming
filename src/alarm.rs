use std::cmp::Ordering;
use chrono::NaiveDateTime;

pub struct Alarm {
    pub when: NaiveDateTime,
    pub what: String,
    pub channel_id: String,
    pub message_id: String
}

impl PartialOrd for Alarm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        NaiveDateTime::partial_cmp(&self.when, &other.when)
    }
}

impl PartialEq for Alarm {
    fn eq(&self, other: &Self) -> bool {
        NaiveDateTime::eq(&self.when, &other.when)
    }
}

impl Eq for Alarm {}

impl Ord for Alarm {
    fn cmp(&self, other: &Self) -> Ordering {
        NaiveDateTime::cmp(&self.when, &other.when)
    }
}
