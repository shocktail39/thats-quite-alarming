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
    let alarm_heap = match file::load() {
        Ok(heap) => heap,
        Err(message) => {
            println!("alarm heap failed to load.\n{message}\nquitting.");
            return;
        }
    };
    let alarm_heap = Arc::new(Mutex::new(alarm_heap));
    let listener_handle = event_listener::start_listening(alarm_heap.clone());
    while !listener_handle.is_finished() {
        let maybe_alarm = {
            let Ok(mut heap_lock) = alarm_heap.lock() else {
                println!("main loop: alarm_heap mutex has been poisoned.  ending.");
                return;
            };
            heap_lock.pop_if_timeup(&Utc::now().naive_utc())
        };
        if let Some(alarm) = maybe_alarm {
            const ALARM_CLOCK: &str = "%E2%8F%B0";
            if let Err(what_happened) = stoat_api::react(&alarm.channel_id, &alarm.message_id, ALARM_CLOCK) {
                println!("main loop: {}\ncaused by reacting to {:?}", what_happened, &alarm);
            }
            if let Err(what_happened) = stoat_api::post_alarm(&alarm) {
                println!("main loop: {}\ncaused by reacting to {:?}", what_happened, &alarm);
            }
            if let Err(what_happened) = file::delete(&alarm) {
                println!("main loop: {}", what_happened);
            }
        } else {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}
