use std::fmt;

use crate::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList {
            top_index: Index::new(0),
            items: vec![],
        }
    }

    pub fn push(&mut self, description: Description, tags: Vec<Tag>) -> TodoItem {
        let item = TodoItem::new(self.top_index, description, tags, false);
        self.items.push(item.clone());
        self.top_index = Index::new(self.top_index.value() + 1);
        item
    }

    pub fn done_with_index(&mut self, idx: Index) -> Option<Index> {
        let item = self.items.get_mut(idx.value() as usize)?;
        item.done = true;
        Some(idx)
    }

    pub fn search(&self, sp: SearchParams) -> Vec<&TodoItem> {
        self.items
            .iter()
            .filter(|item| {
                if item.done {
                    return false;
                }

                for word in sp.words.iter() {
                    if item.description.value().contains(&word.0) {
                        return true;
                    }
                }
                for tag in sp.tags.iter() {
                    if item.tags.contains(tag) {
                        return true;
                    }
                }
                false
            })
            .collect()
    }
}
