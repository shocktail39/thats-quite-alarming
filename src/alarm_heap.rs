use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::alarm::Alarm;

#[derive(Default)]
pub struct AlarmHeap (
    BinaryHeap<Reverse<Alarm>>
);

impl AlarmHeap {
    pub fn push(&mut self, alarm: Alarm) {
        self.0.push(Reverse(alarm));
    }

    pub fn pop(&mut self) -> Option<Alarm> {
        self.0.pop().map(|Reverse(alarm)| alarm)
    }

    pub fn peek(&self) -> Option<&Alarm> {
        self.0.peek().map(|Reverse(alarm)| alarm)
    }
}
