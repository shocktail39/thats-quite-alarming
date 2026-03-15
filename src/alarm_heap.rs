use std::collections::BinaryHeap;

use chrono::NaiveDateTime;

use crate::alarm::Alarm;

#[derive(Debug, Default)]
pub struct AlarmHeap (
    BinaryHeap<Alarm>
);

impl AlarmHeap {
    pub fn push(&mut self, alarm: Alarm) {
        self.0.push(alarm);
    }

    pub fn pop_if_timeup(&mut self, now: &NaiveDateTime) -> Option<Alarm> {
        if self.0.peek().is_some_and(|next_alarm| &next_alarm.when <= now) {
            self.0.pop()
        } else {
            None
        }
    }
}
