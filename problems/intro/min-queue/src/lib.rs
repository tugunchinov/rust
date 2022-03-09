#![forbid(unsafe_code)]

use std::collections::VecDeque;

pub struct MinStack<T> {
    data: VecDeque<T>,
    mins: VecDeque<T>,
}
pub struct MinQueue<T> {
    s1: MinStack<T>,
    s2: MinStack<T>,
    front: Option<T>,
}

impl<T: Clone + Ord> Default for MinStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Ord> MinStack<T> {
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
            mins: VecDeque::new(),
        }
    }

    pub fn push(&mut self, val: T) {
        let min = if self.mins.is_empty() {
            val.clone()
        } else {
            std::cmp::min(self.mins.back().unwrap().clone(), val.clone())
        };

        self.data.push_back(val);
        self.mins.push_back(min);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.mins.pop_back();
        self.data.pop_back()
    }

    pub fn back(&self) -> Option<&T> {
        self.data.back()
    }

    pub fn min(&self) -> Option<&T> {
        self.mins.back()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T: Clone + Ord> Default for MinQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Ord> MinQueue<T> {
    pub fn new() -> Self {
        Self {
            s1: MinStack::new(),
            s2: MinStack::new(),
            front: None,
        }
    }

    pub fn push(&mut self, val: T) {
        self.s1.push(val.clone());

        if self.front.is_none() {
            self.front = Some(val);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.s2.is_empty() {
            if self.s1.is_empty() {
                return None;
            };

            while !self.s1.is_empty() {
                self.s2.push(self.s1.pop().unwrap());
            }
        };

        let to_return = self.s2.pop();

        if self.s2.is_empty() {
            while !self.s1.is_empty() {
                self.s2.push(self.s1.pop().unwrap());
            }
        }

        self.front = self.s2.back().cloned();

        to_return
    }

    pub fn front(&self) -> Option<&T> {
        self.front.as_ref()
    }

    pub fn min(&self) -> Option<&T> {
        if self.s1.is_empty() || self.s2.is_empty() {
            if self.s1.is_empty() {
                self.s2.min()
            } else {
                self.s1.min()
            }
        } else {
            std::cmp::min(self.s1.min(), self.s2.min())
        }
    }

    pub fn len(&self) -> usize {
        self.s1.len() + self.s2.len()
    }

    pub fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
}
