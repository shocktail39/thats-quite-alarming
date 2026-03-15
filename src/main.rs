pub mod alarm;
pub mod alarm_heap;
pub mod config;
pub mod event_listener;
pub mod file;
pub mod json;
pub mod stoat_api;

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use chrono::Utc;

fn main() {
    let alarm_heap = Arc::new(Mutex::new(file::load()));
    let listener_handle = event_listener::start_listening(alarm_heap.clone());
    while !listener_handle.is_finished() {
        let maybe_alarm = {
            let mut heap_lock = alarm_heap.lock().unwrap();
            heap_lock.pop_if_timeup(&Utc::now().naive_utc())
        };
        if let Some(alarm) = maybe_alarm {
            const ALARM_CLOCK: &str = "%E2%8F%B0";
            stoat_api::react(&alarm.channel_id, &alarm.message_id, ALARM_CLOCK);
            stoat_api::post_alarm(&alarm);
            file::delete(&alarm);
        } else {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}
