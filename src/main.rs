pub mod alarm;
pub mod alarm_heap;
pub mod config;
pub mod event_listener;
pub mod json;
pub mod stoat_api;

use std::collections::HashMap;

use crate::alarm::Alarm;
use crate::alarm_heap::AlarmHeap;
use crate::json::Value;
use chrono::Days;
use chrono::Utc;

fn main() {
    event_listener::learn_how_to_websocket(config::BOT_TOKEN);
    // basic testing to make sure the heap pops in the correct order
    /*let now = Utc::now().naive_utc();
    let now_alarm = Alarm::from_message(HashMap::from([
        ("channel".into(), Value::String("some_channel_id".into())),
        ("_id".into(), Value::String("my_user_id".into())),
        ("content".into(), Value::String("@thatsquitealarming in 1s feed dog".into()))
    ])).unwrap();
    let yesterday_alarm = Alarm {
        when: now.checked_sub_days(Days::new(1)).unwrap(),
        what: "yesterday".into(),
        message_id: "c".into(),
        channel_id: "d".into()
    };
    let tomorrow_alarm = Alarm::from_message(HashMap::from([
        ("channel".into(), Value::String("some_channel_id".into())),
        ("_id".into(), Value::String("my_user_id".into())),
        ("content".into(), Value::String("@thatsquitealarming in 1d10h1m30s water garden".into()))
    ])).unwrap();
    let mut alarm_heap = AlarmHeap::default();
    alarm_heap.push(now_alarm);
    alarm_heap.push(yesterday_alarm);
    alarm_heap.push(tomorrow_alarm);
    println!("{:?}", alarm_heap);
    std::thread::sleep(std::time::Duration::from_secs(2));
    let now = Utc::now().naive_utc();
    // this should print the alarm from yesterday
    println!("{}", stoat_api::into_post_message(alarm_heap.pop_if_timeup(&now).unwrap(), config::BOT_TOKEN));
    println!("\n");
    // this should print the alarm for right now
    println!("{}", stoat_api::into_post_message(alarm_heap.pop_if_timeup(&now).unwrap(), config::BOT_TOKEN));
    println!("\n");
    // this should panic, since it is not tomorrow yet
    println!("{}", stoat_api::into_post_message(alarm_heap.pop_if_timeup(&now).unwrap(), config::BOT_TOKEN));*/
}
