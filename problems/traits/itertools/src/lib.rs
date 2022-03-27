#![forbid(unsafe_code)]

pub struct LazyCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    it: std::iter::Fuse<I>,
    cont: Vec<I::Item>,
    idx: usize,
}

impl<I: Iterator> Iterator for LazyCycle<I>
where
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.it.next() {
            self.cont.push(item.clone());
            Some(item)
        } else if !self.cont.is_empty() {
            self.idx += 1;
            Some(self.cont[(self.idx - 1) % self.cont.len()].clone())
        } else {
            None
        }
    }
}
////////////////////////////////////////////////////////////////////////////////

pub struct Extract<I: Iterator> {
    cont: Vec<I::Item>,
}

impl<I: Iterator> Iterator for Extract<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.cont.is_empty() {
            Some(self.cont.remove(0))
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};

pub struct Tee<I>
where
    I: Iterator,
    I::Item: Clone,
{
    common_it: Rc<RefCell<I>>,
    common_cont: Rc<RefCell<Vec<I::Item>>>,
    common_idx: Rc<RefCell<usize>>,
    it_done: Rc<RefCell<bool>>,
    my_idx: usize,
}

impl<I: Iterator> Iterator for Tee<I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.my_idx < *self.common_idx.deref().borrow() {
            if !self.common_cont.deref().borrow().is_empty() {
                self.my_idx += 1;
                Some(self.common_cont.deref().borrow_mut().remove(0))
            } else {
                None
            }
        } else if !*self.it_done.deref().borrow() {
            if let Some(item) = self.common_it.deref().borrow_mut().next() {
                *self.common_idx.deref().borrow_mut() += 1;
                self.my_idx = *self.common_idx.deref().borrow();
                self.common_cont.deref().borrow_mut().push(item.clone());
                Some(item)
            } else {
                *self.it_done.deref().borrow_mut() = true;
                None
            }
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct GroupBy<I, F, V>
where
    I: Iterator,
    F: FnMut(&I::Item) -> V,
    V: Eq,
{
    items: Vec<I::Item>,
    func: F,
}

impl<I: Iterator, F: FnMut(&I::Item) -> V, V: Eq> Iterator for GroupBy<I, F, V> {
    type Item = (V, Vec<I::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.items.is_empty() {
            let mut group = vec![self.items.remove(0)];
            let result = (self.func)(&group[0]);
            while !self.items.is_empty() {
                if result == (self.func)(&self.items[0]) {
                    group.push(self.items.remove(0));
                } else {
                    break;
                }
            }

            Some((result, group))
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait ExtendedIterator: Iterator {
    fn lazy_cycle(self) -> LazyCycle<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        LazyCycle {
            it: self.fuse(),
            cont: Vec::new(),
            idx: 0,
        }
    }

    fn extract(self, index: usize) -> (Option<Self::Item>, Extract<Self>)
    where
        Self: Sized,
    {
        let mut cont: Vec<Self::Item> = self.collect();

        if index < cont.len() {
            (Some(cont.remove(index)), Extract { cont })
        } else {
            (None, Extract { cont })
        }
    }

    fn tee(self) -> (Tee<Self>, Tee<Self>)
    where
        Self: Sized,
        Self::Item: Clone,
    {
        let common_it = Rc::new(RefCell::new(self));
        let common_cont = Rc::new(RefCell::new(Vec::<Self::Item>::new()));
        let common_idx = Rc::new(RefCell::new(0usize));
        let it_done = Rc::new(RefCell::new(false));

        (
            Tee {
                common_it: common_it.clone(),
                common_cont: common_cont.clone(),
                common_idx: common_idx.clone(),
                it_done: it_done.clone(),
                my_idx: 0usize,
            },
            Tee {
                common_it,
                common_cont,
                common_idx,
                it_done,
                my_idx: 0usize,
            },
        )
    }

    fn group_by<F, V>(self, func: F) -> GroupBy<Self, F, V>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> V,
        V: Eq,
    {
        GroupBy {
            items: self.collect(),
            func,
        }
    }
}

impl<I: Iterator> ExtendedIterator for I {}
