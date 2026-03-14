pub mod alarm;
pub mod alarm_heap;
pub mod config;
pub mod event_listener;
pub mod json;
pub mod stoat_api;

use std::sync::Arc;
use std::sync::Mutex;

use chrono::Utc;

use crate::alarm_heap::AlarmHeap;

fn main() {
    let alarm_heap = Arc::new(Mutex::new(AlarmHeap::default()));
    let listener_handle = event_listener::start_listening(alarm_heap.clone());
    while !listener_handle.is_finished() {
        let maybe_alarm = {
            let mut heap_lock = alarm_heap.lock().unwrap();
            heap_lock.pop_if_timeup(&Utc::now().naive_utc())
        };
        if let Some(alarm) = maybe_alarm {
            let channel_id = alarm.channel_id.clone();
            let original_message = alarm.message_id.clone();
            std::thread::spawn(move || {
                const ALARM_CLOCK: &str = "%E2%8F%B0";
                stoat_api::react(&channel_id, &original_message, ALARM_CLOCK);
            });
            std::thread::sleep(config::TIME_BETWEEN_REQUESTS);
            std::thread::spawn(|| {
                stoat_api::post_alarm(alarm);
            });
        }
        std::thread::sleep(config::TIME_BETWEEN_REQUESTS);
    }
}
