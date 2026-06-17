use std::collections::HashMap;

use crate::util::identifier::Identifier;

pub struct Registry<T> {
    map: HashMap<Identifier, T>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(&mut self, identifier: Identifier, data: T) {
        self.map.insert(identifier, data);
    }

    pub fn get(&self, identifier: &Identifier) -> Option<&T> {
        self.map.get(&identifier)
    }
}
