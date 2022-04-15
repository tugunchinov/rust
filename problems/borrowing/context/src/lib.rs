#![forbid(unsafe_code)]

use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Context {
    objects: HashMap<(TypeId, String), Box<dyn Any>>,
    singletones: Vec<Box<dyn Any>>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            singletones: Vec::new(),
        }
    }

    pub fn insert<S: Into<String>, T: Any>(&mut self, key: S, obj: T) {
        self.objects
            .insert((obj.type_id(), key.into()), Box::new(obj));
    }

    pub fn get<'a, T: Any>(&self, key: &'a str) -> &T {
        self.objects[&(TypeId::of::<T>(), key.into())]
            .downcast_ref()
            .unwrap()
    }

    pub fn insert_singletone<T: Any>(&mut self, val: T) {
        for obj in self.singletones.iter_mut() {
            if (&**obj).type_id() == TypeId::of::<T>() {
                *obj = Box::new(val);
                return;
            }
        }
        self.singletones.push(Box::new(val));
    }

    pub fn get_singletone<T: Any>(&self) -> &T {
        for obj in self.singletones.iter() {
            if (&**obj).type_id() == TypeId::of::<T>() {
                return obj.downcast_ref().unwrap();
            }
        }

        panic!()
    }
}
