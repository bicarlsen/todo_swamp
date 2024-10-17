use crate::*;
use rayon::prelude::*;
use std::fmt;
use std::hash::Hash;

type IndexMap<K> = std::collections::HashMap<K, Vec<u64>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index(u64);

impl Index {
    pub fn new(i: u64) -> Index {
        Index(i)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(String);

impl Description {
    pub fn new(s: &str) -> Description {
        Description(s.to_owned())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag(String);

impl Tag {
    pub fn new(s: &str) -> Tag {
        Tag(s.to_owned())
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn from_strings(ss: Vec<&str>) -> Vec<Tag> {
        ss.clone().into_iter().map(|s| Tag::new(s)).collect()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    pub index: Index,
    pub description: Description,
    pub tags: Vec<Tag>,
    pub done: bool,
}

impl TodoItem {
    pub fn new(index: Index, description: Description, tags: Vec<Tag>, done: bool) -> TodoItem {
        TodoItem {
            index,
            description,
            tags,
            done,
        }
    }
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tags = self
            .tags
            .iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, r#"{} "{}" {}"#, self.index, self.description, tags)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoList {
    top_index: Index,
    items: Vec<TodoItem>,

    /// Map of tag to index of active items with that tag.
    tags_index: IndexMap<Tag>,

    /// Map of word to index of active items with that word.
    word_index: IndexMap<String>,
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList {
            top_index: Index::new(0),
            items: vec![],
            tags_index: IndexMap::new(),
            word_index: IndexMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> TodoList {
        TodoList {
            top_index: Index::new(0),
            items: Vec::with_capacity(capacity),
            tags_index: IndexMap::new(),
            word_index: IndexMap::new(),
        }
    }

    pub fn push(&mut self, description: Description, tags: Vec<Tag>) -> TodoItem {
        for tag in tags.iter() {
            let entry = self.tags_index.entry(tag.clone()).or_insert(vec![]);
            entry.push(self.top_index.value());
        }

        for word in description.value().split(" ") {
            let entry = self.word_index.entry(word.to_string()).or_insert(vec![]);
            entry.push(self.top_index.value());
        }

        let item = TodoItem::new(self.top_index, description, tags, false);
        self.items.push(item.clone());
        self.top_index = Index::new(self.top_index.value() + 1);

        item
    }

    pub fn done_with_index(&mut self, idx: Index) -> Option<Index> {
        let item = self.items.get_mut(idx.value() as usize)?;
        if item.done {
            return Some(idx);
        }

        item.done = true;

        for tag in item.tags.iter() {
            let indices = self.tags_index.get_mut(tag).unwrap();
            let idx = indices
                .iter()
                .position(|id| *id == item.index.value())
                .unwrap();

            indices.swap_remove(idx);
            if indices.is_empty() {
                self.tags_index.remove(tag).unwrap();
            }
        }

        for word in item.description.value().split(" ") {
            let indices = self.word_index.get_mut(word).unwrap();
            let idx = indices
                .iter()
                .position(|id| *id == item.index.value())
                .unwrap();

            indices.swap_remove(idx);
            if indices.is_empty() {
                self.word_index.remove(word).unwrap();
            }
        }

        Some(idx)
    }

    pub fn search(&self, sp: SearchParams) -> Vec<&TodoItem> {
        if self.items.len() < 1500 {
            self.search_iter(sp)
        } else {
            self.search_with_index(sp)
        }
    }

    /// Searches iterating over items.
    pub fn search_iter(&self, sp: SearchParams) -> Vec<&TodoItem> {
        self.items
            .iter()
            .filter(|item| {
                if item.done {
                    return false;
                }

                if sp.tags.iter().any(|tag| item.tags.contains(tag)) {
                    return true;
                }

                sp.words
                    .iter()
                    .any(|word| contains_word(&word.0, item.description.value()))
            })
            .collect()
    }

    /// Searches utilizing the tags index.
    pub fn search_with_index(&self, sp: SearchParams) -> Vec<&TodoItem> {
        let mut matches = self.search_index_tags(&sp.tags);
        matches.extend(self.search_index_words(&sp.words));

        matches.sort();
        matches.dedup();
        matches
            .iter()
            .filter_map(|idx| {
                let item = &self.items[**idx as usize];
                if item.done {
                    None
                } else {
                    Some(item)
                }
            })
            .collect()
    }

    /// Filters items by tag.
    /// Returns indices of items that match at least one tag.
    ///
    /// # Returns
    /// Item indices.
    fn search_index_tags(&self, search: &Vec<Tag>) -> Vec<&u64> {
        if search.is_empty() {
            return vec![];
        }

        search
            .par_iter()
            .filter_map(|tag| self.tags_index.get(tag))
            .flatten()
            .collect()
    }

    /// Filters items by word.
    /// Returns indices of items that match at least one word.
    ///
    /// # Returns
    /// Item indices.
    fn search_index_words(&self, search: &Vec<SearchWord>) -> Vec<&u64> {
        if search.is_empty() {
            return vec![];
        }

        self.word_index
            .par_iter()
            .filter_map(|(key, values)| {
                if search.iter().any(|target| matches_word(&target.0, key)) {
                    Some(values)
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

/// # Returns
/// Whether the haystack contains the target as a substring.
fn contains_word(target: impl AsRef<str>, haystack: impl AsRef<str>) -> bool {
    haystack
        .as_ref()
        .split(" ")
        .any(|word| matches_word(&target, word))
}

/// # Returns
/// Whether the word matches the target as a substring.
fn matches_word(target: impl AsRef<str>, word: impl AsRef<str>) -> bool {
    let mut target_chars = target.as_ref().chars().peekable();
    let mut word_chars = word.as_ref().chars().peekable();
    while let Some(target_char) = target_chars.next() {
        while let Some(word_char) = word_chars.next() {
            if word_char == target_char {
                if target_chars.peek().is_none() {
                    return true;
                } else {
                    break;
                }
            }
        }

        if word_chars.peek().is_none() {
            return false;
        }
    }
    unreachable!();
}
