use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::alarm::Alarm;

pub struct AlarmHeap {
    heap: BinaryHeap<Reverse<Alarm>>
}

impl AlarmHeap {
    fn push(&mut self, alarm: Alarm) {
        self.heap.push(Reverse(alarm));
    }

    fn pop(&mut self) -> Option<Alarm> {
        match self.heap.pop() {
            Some(Reverse(alarm)) => Some(alarm),
            None => None
        }
    }

    fn peek(&self) -> Option<&Alarm> {
        match self.heap.peek() {
            Some(Reverse(alarm)) => Some(alarm),
            None => None
        }
    }
}
