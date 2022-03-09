#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

#[derive(Debug)]
pub struct LRUCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    last_time: usize,
    times: HashMap<K, usize>,
    cache_ordered: BTreeMap<usize, K>,
}

impl<K: Clone + Hash + Ord, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        if capacity == 0 {
            panic!("Oh, no!");
        }

        Self {
            capacity,
            cache: HashMap::<K, V>::with_capacity(capacity),
            last_time: 0usize,
            times: HashMap::<K, usize>::with_capacity(capacity),
            cache_ordered: BTreeMap::<usize, K>::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let value = self.cache.get(key)?;
        self.last_time += 1;
        let time = self.times.insert(key.clone(), self.last_time).unwrap();
        let old_key = self.cache_ordered.remove(&time).unwrap();
        self.cache_ordered.insert(self.last_time, old_key);
        Some(value)
    }

    pub fn insert_old(&mut self, key: K, value: V) -> Option<V> {
        let time = self.times.insert(key.clone(), self.last_time);
        match time {
            Some(time) => {
                let old_key = self.cache_ordered.remove(&time).unwrap();
                self.cache_ordered.insert(self.last_time, old_key);
            }
            None => {
                self.cache_ordered.insert(self.last_time, key.clone());
            }
        };

        self.cache_ordered.insert(self.last_time, key.clone());
        self.cache.insert(key, value)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.last_time += 1;
        if self.cache.contains_key(&key) || self.cache.len() < self.capacity {
            self.insert_old(key, value)
        } else {
            let time: usize;
            {
                let (old_time, old_key) = self.cache_ordered.iter().next().unwrap();
                self.cache.remove(old_key);
                self.times.remove(old_key);
                time = *old_time;
            }
            self.cache_ordered.remove(&time);

            self.times.insert(key.clone(), self.last_time);
            self.cache_ordered.insert(self.last_time, key.clone());
            self.cache.insert(key, value)
        }
    }
}
