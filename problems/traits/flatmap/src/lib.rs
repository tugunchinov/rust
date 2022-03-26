#![forbid(unsafe_code)]

use std::{borrow::Borrow, iter::FromIterator, ops::Index};

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, PartialEq, Eq)]
pub struct FlatMap<K, V>(Vec<(K, V)>);

impl<K: Ord, V> FlatMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn as_slice(&self) -> &[(K, V)] {
        self.0.as_slice()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.0.binary_search_by_key(&&key, |(k, _v)| k) {
            Ok(i) => Some(std::mem::replace::<(K, V)>(&mut self.0[i], (key, value)).1),

            Err(i) => {
                self.0.insert(i, (key, value));
                None
            }
        }
    }

    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        match self.0.binary_search_by_key(&key, |(k, _v)| k.borrow()) {
            Ok(i) => Some(&self.0.get(i)?.1),

            Err(_) => None,
        }
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
        match self.0.binary_search_by_key(&key, |(k, _v)| k.borrow()) {
            Ok(i) => Some(self.0.remove(i)),

            Err(_) => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<K: Ord, Q: Ord + ?Sized, V> Index<&Q> for FlatMap<K, V>
where
    K: Borrow<Q>,
{
    type Output = V;
    fn index(&self, index: &Q) -> &V {
        self.get(index).unwrap()
    }
}

impl<K: Ord, V> Extend<(K, V)> for FlatMap<K, V> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<K: Ord, V> From<Vec<(K, V)>> for FlatMap<K, V> {
    fn from(vec: Vec<(K, V)>) -> Self {
        let mut flat_map = FlatMap::<K, V>::new();
        for (key, value) in vec {
            flat_map.insert(key, value);
        }
        flat_map
    }
}

impl<K: Ord, V> From<FlatMap<K, V>> for Vec<(K, V)> {
    fn from(flat_map: FlatMap<K, V>) -> Self {
        flat_map.0
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for FlatMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut flat_map = FlatMap::<K, V>::new();
        for (key, value) in iter {
            flat_map.insert(key, value);
        }
        flat_map
    }
}

impl<K: Ord, V> IntoIterator for FlatMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
