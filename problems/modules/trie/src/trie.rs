#![forbid(unsafe_code)]
#![allow(clippy::borrowed_box)]
use crate::trie_key::ToKeyIter;
use std::{borrow::Borrow, collections::HashMap, hash::Hash, ops::Index};

struct TrieNode<K, V> {
    next: HashMap<K, Box<TrieNode<K, V>>>,
    value: Option<V>,
    is_terminal: bool,
}

impl<K, V> TrieNode<K, V> {
    pub fn new() -> Self {
        Self {
            next: HashMap::new(),
            value: None,
            is_terminal: false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Trie<K: ToKeyIter, V> {
    root: Box<TrieNode<K::Item, V>>,
    len: usize,
}

impl<K: ToKeyIter, V> Trie<K, V>
where
    K::Item: Clone + Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            root: Box::new(TrieNode::new()),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn get_node<'a, Q: 'a + ToKeyIter + ?Sized>(
        &self,
        key: &'a Q,
    ) -> Option<&Box<TrieNode<<K as ToKeyIter>::Item, V>>>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let mut current_node = &self.root;
        let mut key_iter = key.key_iter();
        loop {
            if let Some(next) = key_iter.next() {
                let next_key = next.into();
                current_node = current_node.next.get(&next_key)?;
            } else {
                return Some(current_node);
            };
        }
    }

    fn get_node_mut<'a, Q: 'a + ToKeyIter + ?Sized>(
        &mut self,
        key: &'a Q,
    ) -> Option<&mut Box<TrieNode<<K as ToKeyIter>::Item, V>>>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let mut current_node = &mut self.root;
        let mut key_iter = key.key_iter();
        loop {
            if let Some(next) = key_iter.next() {
                let next_key = next.into();
                current_node = current_node.next.get_mut(&next_key)?;
            } else {
                return Some(current_node);
            };
        }
    }

    fn get_or_create_node<'a, Q: 'a + ToKeyIter + ?Sized>(
        &mut self,
        key: &'a Q,
    ) -> &mut Box<TrieNode<<K as ToKeyIter>::Item, V>>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let mut current_node = &mut self.root;
        let mut key_iter = key.key_iter();
        loop {
            if let Some(next) = key_iter.next() {
                let next_key = next.into();

                if !current_node.next.contains_key(&next_key) {
                    current_node
                        .next
                        .insert(next_key.clone(), Box::new(TrieNode::new()));
                }

                current_node = current_node.next.get_mut(&next_key).unwrap();
            } else {
                return current_node;
            };
        }
    }

    pub fn insert<'a, Q: 'a + ToKeyIter + ?Sized>(&mut self, key: &'a Q, value: V) -> Option<V>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let node = self.get_or_create_node(key);
        let ret_val = std::mem::replace(&mut node.value, Some(value));
        node.is_terminal = true;
        if ret_val.is_none() {
            self.len += 1;
        }
        ret_val
    }

    pub fn get<'a, Q: 'a + ToKeyIter + ?Sized>(&self, key: &'a Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let node = self.get_node(key)?;
        node.value.as_ref()
    }

    pub fn get_mut<'a, Q: 'a + ToKeyIter + ?Sized>(&mut self, key: &'a Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let node = self.get_node_mut(key)?;
        node.value.as_mut()
    }

    pub fn contains<'a, Q: 'a + ToKeyIter + ?Sized>(&self, key: &'a Q) -> bool
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        self.get(key).is_some()
    }

    fn has_terminal<'a, Q: 'a + ToKeyIter + ?Sized>(
        &self,
        node: &Box<TrieNode<<K as ToKeyIter>::Item, V>>,
    ) -> bool
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        if node.is_terminal {
            true
        } else {
            for next in node.next.values() {
                if self.has_terminal(next) {
                    return true;
                }
            }
            false
        }
    }

    pub fn starts_with<'a, Q: 'a + ToKeyIter + ?Sized>(&self, key: &'a Q) -> bool
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        if let Some(node) = self.get_node(key) {
            self.has_terminal(node)
        } else {
            false
        }
    }

    pub fn remove<'a, Q: 'a + ToKeyIter + ?Sized>(&mut self, key: &'a Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q::KeyIter<'a>: Iterator,
        <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
    {
        let mut node = self.get_node_mut(key)?;
        node.is_terminal = false;
        let ret_val = std::mem::replace(&mut node.value, None);
        if ret_val.is_some() {
            self.len -= 1;
        }
        ret_val
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'a, K: ToKeyIter, Q: ?Sized, V> Index<&'a Q> for Trie<K, V>
where
    K::Item: Clone + Hash + Eq,
    K: Borrow<Q>,
    Q: 'a + ToKeyIter,
    Q::KeyIter<'a>: Iterator,
    <K as ToKeyIter>::Item: From<<<Q as ToKeyIter>::KeyIter<'a> as Iterator>::Item>,
{
    type Output = V;

    fn index(&self, key: &'a Q) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K: ToKeyIter, V> Default for Trie<K, V>
where
    K::Item: Clone + Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}
