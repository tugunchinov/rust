#![forbid(unsafe_code)]
#![allow(clippy::needless_lifetimes)]
use std::str::Chars;

pub trait ToKeyIter {
    type Item;
    type KeyIter<'a>
    where
        Self: 'a;

    fn key_iter<'a>(&'a self) -> Self::KeyIter<'a>;
}

impl ToKeyIter for str {
    type Item = char;
    type KeyIter<'a> = Chars<'a>;

    fn key_iter<'a>(&'a self) -> Self::KeyIter<'a> {
        self.chars()
    }
}

impl ToKeyIter for String {
    type Item = char;
    type KeyIter<'a> = Chars<'a>;

    fn key_iter<'a>(&'a self) -> Self::KeyIter<'a> {
        self.chars()
    }
}

////////////////////////////////////////////////////////////////////////////////

// Bonus

// pub trait FromKeyIter {
//     fn to_key(self) -> ???;
// }

// impl FromKeyIter for ???
// TODO: your code goes here.

////////////////////////////////////////////////////////////////////////////////

// Bonus

// pub trait TrieKey
// TODO: your code goes here.
