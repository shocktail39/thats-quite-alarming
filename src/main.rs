pub mod alarm;
pub mod alarm_heap;
pub mod config;
pub mod stoat_api;

use crate::alarm::Alarm;
use crate::alarm_heap::AlarmHeap;
use chrono::Days;
use chrono::Utc;

fn main() {
    // basic testing to make sure the heap pops in the correct order
    let now_alarm = Alarm {
        when: Utc::now().naive_utc(),
        what: r#"right \now""#.into(),
        message_id: "a".into(),
        channel_id: "b".into()
    };
    let yesterday_alarm = Alarm {
        when: Utc::now().naive_utc().checked_sub_days(Days::new(1)).unwrap(),
        what: "yesterday".into(),
        message_id: "c".into(),
        channel_id: "d".into()
    };
    let tomorrow_alarm = Alarm {
        when: Utc::now().naive_utc().checked_add_days(Days::new(1)).unwrap(),
        what: "tomorrow".into(),
        message_id: "e".into(),
        channel_id: "f".into()
    };
    let mut alarm_heap = AlarmHeap::default();
    alarm_heap.push(now_alarm);
    alarm_heap.push(yesterday_alarm);
    alarm_heap.push(tomorrow_alarm);
    println!("{}", stoat_api::into_post_message(alarm_heap.pop().unwrap(), config::BOT_TOKEN));
    println!("\n");
    println!("{}", stoat_api::into_post_message(alarm_heap.pop().unwrap(), config::BOT_TOKEN));
    println!("\n");
    println!("{}", stoat_api::into_post_message(alarm_heap.pop().unwrap(), config::BOT_TOKEN));
}
