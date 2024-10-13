use std::fmt;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    Add(Description, Vec<Tag>),
    Done(Index),
    Search(SearchParams),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchParams {
    pub words: Vec<SearchWord>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchWord(pub String);
impl SearchWord {
    pub fn new(s: &str) -> SearchWord {
        SearchWord(s.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryResult {
    Added(TodoItem),
    Done,
    Found(Vec<todo_list::TodoItem>),
}

impl fmt::Display for QueryResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            QueryResult::Added(ti) => write!(f, "{}", ti.index),
            QueryResult::Done => write!(f, "done"),
            QueryResult::Found(rs) => {
                let mut sorted = rs.iter().collect::<Vec<_>>();
                sorted.sort_by_key(|r| r.index);

                let mut buff: Vec<String> = vec![];
                buff.push(format!("{} item(s) found", sorted.len()));
                for i in sorted.iter().rev() {
                    buff.push(format!("{}", i));
                }
                write!(f, "{}", buff.join("\n"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError(pub String);

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "An error occurred while processing the query: {}.",
            self.0
        )
    }
}
