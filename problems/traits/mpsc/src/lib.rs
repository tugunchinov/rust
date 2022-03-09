#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("channel is closed")]
pub struct SendError<T> {
    pub value: T,
}

pub struct Sender<T> {
    queue: Rc<RefCell<VecDeque<T>>>,
    closed: Rc<RefCell<bool>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if self.is_closed() {
            Err(SendError { value })
        } else {
            self.queue.borrow_mut().push_back(value);
            Ok(())
        }
    }

    pub fn is_closed(&self) -> bool {
        *self.closed.borrow()
    }

    pub fn same_channel(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.closed, &other.closed)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            queue: Rc::clone(&self.queue),
            closed: Rc::clone(&self.closed),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {}
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("channel is empty")]
    Empty,
    #[error("channel is closed")]
    Closed,
}

pub struct Receiver<T> {
    queue: Rc<RefCell<VecDeque<T>>>,
    closed: Rc<RefCell<bool>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, ReceiveError> {
        if Rc::strong_count(&self.queue) == 1 {
            self.close();
        }

        if !self.queue.borrow().is_empty() {
            Ok(self.queue.borrow_mut().pop_front().unwrap())
        } else if *self.closed.borrow() {
            Err(ReceiveError::Closed)
        } else {
            Err(ReceiveError::Empty)
        }
    }

    pub fn close(&mut self) {
        *self.closed.borrow_mut() = true;
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.close();
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let queue = Rc::new(RefCell::new(VecDeque::<T>::new()));
    let closed = Rc::new(RefCell::new(false));

    let sender = Sender::<T> {
        queue: Rc::clone(&queue),
        closed: Rc::clone(&closed),
    };

    let receiver = Receiver::<T> {
        queue: Rc::clone(&queue),
        closed: Rc::clone(&closed),
    };

    (sender, receiver)
}
