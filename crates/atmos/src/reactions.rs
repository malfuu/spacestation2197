use bevy::{platform::collections::HashMap, prelude::*};

// TODO: placeholder for now
pub type Reaction = ();

#[derive(Resource)]
pub struct ReactionList {
    list: HashMap<String, Reaction>,
}

impl Default for ReactionList {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactionList {
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, reaction: Reaction) {
        self.list.insert(name, reaction);
    }

    pub fn get(&self, name: &str) -> Option<&Reaction> {
        self.list.get(name)
    }
}
