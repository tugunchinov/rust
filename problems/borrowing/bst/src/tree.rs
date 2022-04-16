#![forbid(unsafe_code)]
use std::borrow::Borrow;

use crate::node::Node;

pub struct AVLTreeMap<K, V> {
    root: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> Default for AVLTreeMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> AVLTreeMap<K, V> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn len(&self) -> usize {
        unimplemented!()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        Some(self.get_key_value(key)?.1)
    }

    pub fn get_key_value<Q: Ord + ?Sized>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
    {
        Node::<K, V>::get_key_value(&self.root, key)
    }

    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        Node::<K, V>::insert(&mut self.root, key, val)
    }

    pub fn nth_key_value(&self, _k: usize) -> Option<(&K, &V)> {
        unimplemented!()
    }

    pub fn remove<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        Some(self.remove_entry(key)?.1)
    }

    pub fn remove_entry<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
    {
        let (new_root, removed) = Node::<K, V>::remove_entry(self.root.take(), key);
        self.root = new_root;
        removed
    }
}
