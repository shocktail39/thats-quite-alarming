pub mod alarm;
pub mod alarm_heap;
pub mod config;
pub mod event_listener;
pub mod json;
pub mod stoat_api;

use crate::alarm::Alarm;
use crate::alarm_heap::AlarmHeap;
use chrono::Days;
use chrono::Utc;

fn main() {
    //event_listener::learn_how_to_websocket();
    // basic testing to make sure the heap pops in the correct order
    let now = Utc::now().naive_utc();
    let now_alarm = Alarm {
        when: now,
        what: r#"right \now""#.into(),
        message_id: "a".into(),
        channel_id: "b".into()
    };
    let yesterday_alarm = Alarm {
        when: now.checked_sub_days(Days::new(1)).unwrap(),
        what: "yesterday".into(),
        message_id: "c".into(),
        channel_id: "d".into()
    };
    let tomorrow_alarm = Alarm {
        when: now.checked_add_days(Days::new(1)).unwrap(),
        what: "tomorrow".into(),
        message_id: "e".into(),
        channel_id: "f".into()
    };
    let mut alarm_heap = AlarmHeap::default();
    alarm_heap.push(now_alarm);
    alarm_heap.push(yesterday_alarm);
    alarm_heap.push(tomorrow_alarm);
    // this should print the alarm from yesterday
    println!("{}", stoat_api::into_post_message(alarm_heap.pop_if_timeup(&now).unwrap(), config::BOT_TOKEN));
    println!("\n");
    // this should print the alarm for right now
    println!("{}", stoat_api::into_post_message(alarm_heap.pop_if_timeup(&now).unwrap(), config::BOT_TOKEN));
    println!("\n");
    // this should panic, since it is not tomorrow yet
    println!("{}", stoat_api::into_post_message(alarm_heap.pop_if_timeup(&now).unwrap(), config::BOT_TOKEN));
}
