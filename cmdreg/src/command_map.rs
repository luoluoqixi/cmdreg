use std::{collections::HashMap, ops::Deref};

/// A generic string-keyed map for storing command handlers.
#[derive(Debug, Default)]
pub struct CommandMap<K, F> {
    commands: HashMap<K, F>,
}

impl<K, F> CommandMap<K, F>
where
    K: Eq + std::hash::Hash + AsRef<str>,
{
    pub fn new() -> Self {
        Self {
            commands: Default::default(),
        }
    }

    pub fn reg(&mut self, command: K, f: F) {
        self.commands.insert(command, f);
    }

    pub fn unreg(&mut self, command: &K) {
        self.commands.remove(command);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn get(&self, key: &K) -> Option<&F> {
        self.commands.get(key)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn keys(&self) -> Vec<String> {
        self.commands
            .keys()
            .map(|k| k.as_ref().to_string())
            .collect()
    }
}

impl<K, F> Deref for CommandMap<K, F> {
    type Target = HashMap<K, F>;
    fn deref(&self) -> &Self::Target {
        &self.commands
    }
}
