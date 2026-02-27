use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Duration;
use chrono::NaiveDateTime;
use chrono::Utc;

use crate::json::Value;

#[derive(Debug)]
pub struct Alarm {
    pub when: NaiveDateTime,
    pub what: String,
    pub channel_id: String,
    pub message_id: String
}

impl Alarm {
    fn parse_timer(mut text: String) -> Option<(NaiveDateTime, String)> {
        // messages are formatted like this:
        // "@bot_handle in 2d12h5m30s wish mom a happy birthday"
        // everything before "in " isn't parsed,
        // and then the first word after "in " is converted to a duration,
        // and then the rest of the message the text to say when the alarm goes off.
        let mut timer = text.split_off(text.find("in ").map(|index| index + "in ".len())?);
        let message = timer.split_off(timer.find(" ").map(|index| index + " ".len())?);
        let now = Utc::now().naive_utc();
        let duration = {
            const ONE_SECOND: u64 = 1;
            const ONE_MINUTE: u64 = ONE_SECOND * 60;
            const ONE_HOUR: u64 = ONE_MINUTE * 60;
            const ONE_DAY: u64 = ONE_HOUR * 24;
            let days = if let Some(d_index) = timer.find(['d','D']) {
                let the_rest_of_it = timer.split_off(d_index + 1);
                timer.pop();
                let days = timer.parse::<u64>().ok()?;
                timer = the_rest_of_it;
                days * ONE_DAY
            } else {
                0u64
            };
            let hours = if let Some(h_index) = timer.find(['h','H']) {
                let the_rest_of_it = timer.split_off(h_index + 1);
                timer.pop();
                let hours = timer.parse::<u64>().ok()?;
                timer = the_rest_of_it;
                hours * ONE_HOUR
            } else {
                0u64
            };
            let mins = if let Some(m_index) = timer.find(['m','M']) {
                let the_rest_of_it = timer.split_off(m_index + 1);
                timer.pop();
                let mins = timer.parse::<u64>().ok()?;
                timer = the_rest_of_it;
                mins * ONE_MINUTE
            } else {
                0u64
            };
            let secs = if let Some(s_index) = timer.find(['s','S']) {
                let _ = timer.split_off(s_index);
                let secs = timer.parse::<u64>().ok()?;
                secs * ONE_SECOND
            } else {
                0u64
            };
            Duration::from_secs(days + hours + mins + secs)
        };

        Some((now + duration, message))
    }

    pub fn from_message(mut message: HashMap<String, Value>) -> Option<Self> {
        let Some(Value::String(channel_id)) = message.remove("channel") else {
            return None;
        };
        let Some(Value::String(message_id)) = message.remove("_id") else {
            return None;
        };
        let Some(Value::String(message_text)) = message.remove("content") else {
            return None;
        };
        let (when, what) = Self::parse_timer(message_text)?;

        Some(Self {
            when,
            what,
            channel_id,
            message_id
        })
    }
}

impl PartialOrd for Alarm {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
