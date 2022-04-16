#![forbid(unsafe_code)]
#![allow(clippy::type_complexity)]
use std::borrow::Borrow;

pub struct Node<K, V> {
    pub key: K,
    pub val: V,
    pub height: i32,
    pub left: Option<Box<Node<K, V>>>,
    pub right: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> Node<K, V> {
    pub fn insert(root: &mut Option<Box<Node<K, V>>>, key: K, val: V) -> Option<V> {
        if root.is_none() {
            *root = Some(Box::new(Node {
                key,
                val,
                height: 1,
                left: None,
                right: None,
            }));
            None
        } else if key < root.as_mut()?.key {
            let result = Self::insert(&mut root.as_mut()?.left, key, val);
            root.as_mut()?.update_height();
            root.as_mut()?.rebalance();
            result
        } else if key > root.as_mut()?.key {
            let result = Self::insert(&mut root.as_mut()?.right, key, val);
            root.as_mut()?.update_height();
            root.as_mut()?.rebalance();
            result
        } else {
            Some(std::mem::replace(&mut root.as_mut()?.val, val))
        }
    }

    pub fn get_key_value<'a, Q: Ord + ?Sized>(
        root: &'a Option<Box<Node<K, V>>>,
        key: &Q,
    ) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
    {
        if root.is_none() {
            None
        } else if key < root.as_ref()?.key.borrow() {
            Self::get_key_value(&root.as_ref()?.left, key)
        } else if key > root.as_ref()?.key.borrow() {
            Self::get_key_value(&root.as_ref()?.right, key)
        } else {
            Some((&root.as_ref()?.key, &root.as_ref()?.val))
        }
    }

    pub fn nth_key_value<'a>(
        root: &'a Option<Box<Node<K, V>>>,
        k: &mut usize,
    ) -> Option<(&'a K, &'a V)> {
        if root.is_none() {
            return None;
        }

        let left = Self::nth_key_value(&root.as_ref().unwrap().left, k);

        if left.is_some() {
            return left;
        }

        if *k == 0 {
            return Some((&root.as_ref()?.key, &root.as_ref()?.val));
        } else {
            *k -= 1;
        }

        return Self::nth_key_value(&root.as_ref().unwrap().right, k);
    }

    fn extract_min(
        mut root: Option<Box<Node<K, V>>>,
    ) -> (Option<Box<Node<K, V>>>, Option<Box<Node<K, V>>>) {
        if root.as_mut().unwrap().left.is_some() {
            let (new_root, min) = Self::extract_min(root.as_mut().unwrap().left.take());
            root.as_mut().unwrap().left = new_root;
            root.as_mut().unwrap().update_height();
            root.as_mut().unwrap().rebalance();
            (root, min)
        } else {
            let mut min = root.unwrap();
            (min.right.take(), Some(min))
        }
    }

    pub fn remove_entry<Q: Ord + ?Sized>(
        mut root: Option<Box<Node<K, V>>>,
        key: &Q,
    ) -> (Option<Box<Node<K, V>>>, Option<(K, V)>)
    where
        K: Borrow<Q>,
    {
        if root.is_none() {
            (None, None)
        } else if key < root.as_ref().unwrap().key.borrow() {
            let (left_root, val) = Self::remove_entry(root.as_mut().unwrap().left.take(), key);
            root.as_mut().unwrap().left = left_root;
            root.as_mut().unwrap().update_height();
            root.as_mut().unwrap().rebalance();
            (root, val)
        } else if key > root.as_ref().unwrap().key.borrow() {
            let (right_root, val) = Self::remove_entry(root.as_mut().unwrap().right.take(), key);
            root.as_mut().unwrap().right = right_root;
            root.as_mut().unwrap().update_height();
            root.as_mut().unwrap().rebalance();
            (root, val)
        } else if root.as_ref().unwrap().left.is_some() && root.as_ref().unwrap().right.is_some() {
            let (new_root, min) = Self::extract_min(root.as_mut().unwrap().right.take());
            root.as_mut().unwrap().right = new_root;

            let unwrapped = min.unwrap();

            let old_key = std::mem::replace(&mut root.as_mut().unwrap().key, unwrapped.key);
            let old_val = std::mem::replace(&mut root.as_mut().unwrap().val, unwrapped.val);

            root.as_mut().unwrap().update_height();
            root.as_mut().unwrap().rebalance();

            (root, Some((old_key, old_val)))
        } else if root.as_ref().unwrap().left.is_some() {
            let mut unwrapped = root.unwrap();
            (unwrapped.left.take(), Some((unwrapped.key, unwrapped.val)))
        } else if root.as_ref().unwrap().right.is_some() {
            let mut unwrapped = root.unwrap();
            (unwrapped.right.take(), Some((unwrapped.key, unwrapped.val)))
        } else {
            let unwrapped = root.unwrap();
            (None, Some((unwrapped.key, unwrapped.val)))
        }
    }

    fn update_height(&mut self) {
        self.height = std::cmp::max(
            self.left.as_ref().map_or(0, |node| node.height),
            self.right.as_ref().map_or(0, |node| node.height),
        ) + 1
    }

    fn get_balance_diff(&self) -> i32 {
        let left_height = self.left.as_ref().map_or(0, |node| node.height);
        let right_height = self.right.as_ref().map_or(0, |node| node.height);

        left_height - right_height
    }

    fn rotate_left(&mut self) {
        let mut b = self.right.take().unwrap();
        std::mem::swap(&mut self.key, &mut b.key);
        std::mem::swap(&mut self.val, &mut b.val);
        self.right = b.right.take();
        b.right = std::mem::replace(&mut b.left, self.left.take());
        self.left = Some(b);

        self.left.as_mut().unwrap().update_height();
        self.update_height();
    }

    fn rotate_right(&mut self) {
        let mut b = self.left.take().unwrap();
        std::mem::swap(&mut self.key, &mut b.key);
        std::mem::swap(&mut self.val, &mut b.val);
        self.left = b.left.take();
        b.left = std::mem::replace(&mut b.right, self.right.take());
        self.right = Some(b);

        self.right.as_mut().unwrap().update_height();
        self.update_height();
    }

    fn big_rotate_left(&mut self) {
        self.right.as_mut().unwrap().rotate_right();
        self.rotate_left();
    }

    fn big_rotate_right(&mut self) {
        self.left.as_mut().unwrap().rotate_left();
        self.rotate_right();
    }

    fn rebalance(&mut self) {
        match self.get_balance_diff() {
            -2 => {
                if let Some(right) = self.right.as_mut() {
                    match right.get_balance_diff() {
                        -1 | 0 => self.rotate_left(),

                        1 => {
                            if let Some(left) = right.left.as_mut() {
                                if (-1..=1).contains(&left.get_balance_diff()) {
                                    self.big_rotate_left()
                                }
                            }
                        }

                        _ => {}
                    }
                }
            }

            2 => {
                if let Some(left) = self.left.as_mut() {
                    match left.get_balance_diff() {
                        1 | 0 => self.rotate_right(),

                        -1 => {
                            if let Some(right) = left.right.as_mut() {
                                if (-1..=1).contains(&right.get_balance_diff()) {
                                    self.big_rotate_right()
                                }
                            }
                        }

                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }
}
