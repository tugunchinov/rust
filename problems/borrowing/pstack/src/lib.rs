#![forbid(unsafe_code)]
use std::rc::Rc;

pub struct PRef<T> {
    len: usize,
    val: Rc<T>,
    prev: Option<Rc<PRef<T>>>,
}

impl<T> std::ops::Deref for PRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.val.as_ref()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct PStack<T> {
    head: Option<Rc<PRef<T>>>,
}

impl<T> Default for PStack<T> {
    fn default() -> Self {
        Self { head: None }
    }
}

impl<T> Clone for PStack<T> {
    fn clone(&self) -> Self {
        let head = if self.head.is_some() {
            Some(Rc::clone(self.head.as_ref().unwrap()))
        } else {
            None
        };

        Self { head }
    }
}

impl<T> PStack<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&self, value: T) -> Self {
        let prev = if self.head.is_some() {
            Some(Rc::clone(self.head.as_ref().unwrap()))
        } else {
            None
        };

        let new_node = PRef::<T> {
            len: self.len() + 1,
            val: Rc::new(value),
            prev,
        };

        Self {
            head: Some(Rc::new(new_node)),
        }
    }

    pub fn pop(&self) -> Option<(PRef<T>, Self)> {
        let new_head = if self.head.as_ref()?.prev.is_some() {
            Some(Rc::clone(self.head.as_ref()?.prev.as_ref().unwrap()))
        } else {
            None
        };

        let prev = if self.head.as_ref()?.prev.is_some() {
            Some(Rc::clone(self.head.as_ref()?.prev.as_ref().unwrap()))
        } else {
            None
        };

        let last = PRef::<T> {
            len: self.head.as_ref()?.len,
            val: Rc::clone(&self.head.as_ref()?.val),
            prev,
        };

        Some((last, Self { head: new_head }))
    }

    pub fn len(&self) -> usize {
        if self.head.is_some() {
            self.head.as_ref().unwrap().len
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = PRef<T>> {
        let mut vals = Vec::<PRef<T>>::new();
        if self.head.is_none() {
            return vals.into_iter();
        }

        let mut tmp = Rc::clone(self.head.as_ref().unwrap());
        loop {
            let prev = if tmp.prev.is_some() {
                Some(Rc::clone(tmp.prev.as_ref().unwrap()))
            } else {
                None
            };

            vals.push(PRef::<T> {
                len: tmp.as_ref().len,
                val: Rc::clone(&tmp.as_ref().val),
                prev,
            });

            if tmp.prev.is_none() {
                return vals.into_iter();
            }

            tmp = Rc::clone(tmp.prev.as_ref().unwrap());
        }
    }
}
