use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::alarm::Alarm;

pub struct AlarmHeap (
    BinaryHeap<Reverse<Alarm>>
);

impl Default for AlarmHeap {
    fn default() -> Self {
        AlarmHeap(BinaryHeap::default())
    }
}

impl AlarmHeap {
    pub fn push(&mut self, alarm: Alarm) {
        self.0.push(Reverse(alarm));
    }

    pub fn pop(&mut self) -> Option<Alarm> {
        match self.0.pop() {
            Some(Reverse(alarm)) => Some(alarm),
            None => None
        }
    }

    pub fn peek(&self) -> Option<&Alarm> {
        match self.0.peek() {
            Some(Reverse(alarm)) => Some(alarm),
            None => None
        }
    }
}
