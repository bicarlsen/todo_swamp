use fake::{
    faker::lorem::en::{Word, Words},
    Fake,
};
use rand::prelude::*;
use std::{fmt, fs};
use todo_swamp as todo;

const MAX_INPUT: usize = 5_000_000;

#[test]
fn test_bed() {
    let mut queries = Queries::new();
    assert!(queries.input().is_empty());

    let add = QueryAdd::random();
    queries.push(add.into());
    assert!(queries.todos_at_index(0).is_empty());
    assert_eq!(queries.todos_at_index(1).len(), 1);

    queries.push(QueryDone::new(0).into());
    assert_eq!(queries.todos_at_index(1).len(), 1);
    assert!(queries.todos_at_index(2).is_empty());
    assert!(queries.completed_at_index(1).is_empty());
    assert_eq!(queries.completed_at_index(2).len(), 1);

    let mut rng = rand::thread_rng();
    let add = QueryAdd::random();
    assert!(contains_word(
        add.search_word(&mut rng),
        add.description.value()
    ));

    // CAUTION: May succeed if target happens to be included.
    let target = Word().fake::<String>();
    assert!(!contains_word(target, add.description.value()));

    let desc = "this is the first add";
    let add = QueryAdd::new(desc);
    let add_in = format!(r#"add "{}""#, desc);
    assert_eq!(add.to_string(), add_in);

    let desc = "this is with tags";
    let tags = vec!["first_tag", "second_tag"];
    let tags_in = tags
        .iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<String>>()
        .join(" ");

    let add_tags_in = format!(r#"add "{}" {}"#, desc, tags_in);
    let add_tags = QueryAdd::with_tags(desc, tags);
    assert_eq!(add_tags.to_string(), add_tags_in);

    let mut queries = Queries::new();
    queries.push(add.into());
    queries.push(add_tags.into());

    let input = queries.input();
    assert_eq!(input.len(), 2);
    assert_eq!(input[0], add_in);
    assert_eq!(input[1], add_tags_in);

    let output = queries.output();
    assert_eq!(output.len(), 2);
    assert_eq!(output[0], "0");
    assert_eq!(output[1], "1");

    let add_bread = QueryAdd::with_tags("buy bread", vec!["groceries"]);
    let add_milk = QueryAdd::with_tags("buy milk", vec!["groceries"]);
    let add_parents = QueryAdd::with_tags("call parents", vec!["relatives"]);
    let search_groceries = QuerySearch::tags(vec!["groceries"]);
    let search_buy = QuerySearch::words(vec!["buy"]);
    let search_a = QuerySearch::words(vec!["a"]);
    let done_0 = QueryDone::new(0);
    let done_2 = QueryDone::new(2);

    let queries = Queries::from(vec![
        add_bread.into(),        // add "buy bread" #groceries
        add_milk.into(),         // add "buy milk" #groceries
        add_parents.into(),      // add "call parents" #relatives
        search_groceries.into(), // search #groceries
        search_buy.into(),       // search buy
        search_a.clone().into(), // search a
        done_0.into(),           // done 0
        search_a.clone().into(), // search a
        done_2.into(),           // done 2
        search_a.clone().into(), // search a
    ]);

    let out = vec![
        "0",
        "1",
        "2",
        "2 item(s) found\n1 \"buy milk\" #groceries\n0 \"buy bread\" #groceries",
        "2 item(s) found\n1 \"buy milk\" #groceries\n0 \"buy bread\" #groceries",
        "2 item(s) found\n2 \"call parents\" #relatives\n0 \"buy bread\" #groceries",
        "done",
        "1 item(s) found\n2 \"call parents\" #relatives",
        "done",
        "0 item(s) found",
    ];

    assert_eq!(queries.output(), out);
}

#[test]
fn sample() {
    let mut todos = todo::TodoList::new();
    let expected = fs::read_to_string("tests/fixtures/sample.out").unwrap();
    let input = fs::read_to_string("tests/fixtures/sample.in").unwrap();
    let input = input.split("\n");

    let mut out = Vec::new();
    let mut err = Vec::new();
    input.for_each(|line| todo::runner::run_line(line, &mut todos, &mut out, &mut err));
    assert!(err.is_empty());
    let out = String::from_utf8(out).unwrap();
    assert_eq!(out, expected);
}

#[test]
fn basic() {
    let mut rng = rand::thread_rng();
    let adds = (0..3).map(|_| QueryAdd::random()).collect::<Vec<_>>();
    let searches = adds
        .iter()
        .map(|query| QuerySearch::words(vec![query.search_word(&mut rng)]))
        .collect::<Vec<_>>();

    let queries = adds
        .into_iter()
        .map(|query| query.into())
        .chain(searches.into_iter().map(|query| query.into()))
        .collect();
    let queries = Queries::from(queries);
    let mut expected = queries.output().join("\n");
    expected.push_str("\n");

    let mut todos = todo::TodoList::new();
    let mut out = Vec::new();
    let mut err = Vec::new();
    queries
        .input()
        .iter()
        .for_each(|line| todo::runner::run_line(line, &mut todos, &mut out, &mut err));
    assert!(err.is_empty());
    let out = String::from_utf8(out).unwrap();
    assert_eq!(out, expected);
}

enum Query {
    Add(QueryAdd),
    Done(QueryDone),
    Search(QuerySearch),
}

impl From<QueryAdd> for Query {
    fn from(value: QueryAdd) -> Self {
        Self::Add(value)
    }
}

impl From<QueryDone> for Query {
    fn from(value: QueryDone) -> Self {
        Self::Done(value)
    }
}

impl From<QuerySearch> for Query {
    fn from(value: QuerySearch) -> Self {
        Self::Search(value)
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Query::Add(query) => query.to_string(),
            Query::Done(query) => query.to_string(),
            Query::Search(query) => query.to_string(),
        };

        write!(f, "{}", out)
    }
}

struct Queries {
    /// List of queries.
    queries: Vec<Query>,

    /// List of todos and the index of the query that created it.
    todos: Vec<(usize, todo::TodoItem)>,
}

impl Queries {
    pub fn new() -> Self {
        Self {
            queries: vec![],
            todos: vec![],
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queries: Vec::with_capacity(capacity),
            todos: Vec::with_capacity(capacity / 3),
        }
    }

    pub fn from(queries: Vec<Query>) -> Self {
        let todos = queries
            .iter()
            .enumerate()
            .filter_map(|(idx, query)| {
                if let Query::Add(ref query) = query {
                    let todo = todo::TodoItem::new(
                        todo::Index::new(idx as u64),
                        query.description.clone(),
                        query.tags.clone(),
                        false,
                    );
                    Some((idx, todo))
                } else {
                    None
                }
            })
            .collect();

        Self { queries, todos }
    }

    pub fn push(&mut self, query: Query) {
        if let Query::Add(ref query) = query {
            let item = todo::TodoItem::new(
                todo::Index::new(self.todos.len() as u64),
                query.description.clone(),
                query.tags.clone(),
                false,
            );
            self.todos.push((self.queries.len(), item));
        }

        self.queries.push(query);
    }

    /// Generates the input that should be fed to `stdin`.
    pub fn input(&self) -> Vec<String> {
        self.queries.iter().map(|query| query.to_string()).collect()
    }
}

impl Queries {
    /// Generates the output that should be emitted on `stdout`.
    pub fn output(&self) -> Vec<String> {
        self.queries
            .iter()
            .enumerate()
            .map(|(index, query)| match query {
                Query::Add(_) => index.to_string(),
                Query::Done(_) => "done".to_string(),
                Query::Search(query) => {
                    let items = self
                        .todos_at_index(index)
                        .iter()
                        .filter(|item| {
                            if query.tags.iter().any(|tag| item.tags.contains(tag)) {
                                return true;
                            }

                            if query
                                .words
                                .iter()
                                .any(|word| contains_word(word, item.description.value()))
                            {
                                return true;
                            }

                            false
                        })
                        .map(|item| item.to_string())
                        .rev()
                        .collect::<Vec<_>>();

                    if items.is_empty() {
                        "0 item(s) found".to_string()
                    } else {
                        format!("{} item(s) found\n{}", items.len(), items.join("\n"))
                    }
                }
            })
            .collect()
    }

    /// # Returns
    /// Todos that are **NOT** `done` by the given query, but before the given query has run.
    pub fn todos_at_index(&self, index: usize) -> Vec<&todo::TodoItem> {
        let completed = self
            .queries
            .iter()
            .take(index)
            .filter_map(|query| match query {
                Query::Done(query) => Some(query.index()),
                Query::Add(_) | Query::Search(_) => None,
            })
            .collect::<Vec<_>>();

        self.todos
            .iter()
            .take_while(|(idx, _)| *idx < index)
            .filter_map(|(_, item)| {
                if completed.contains(&&item.index) {
                    None
                } else {
                    Some(item)
                }
            })
            .collect()
    }

    /// # Returns
    /// Todos that **ARE** `done` by the given query, but before the given query has run.
    pub fn completed_at_index(&self, index: usize) -> Vec<&todo::TodoItem> {
        let completed = self
            .queries
            .iter()
            .take(index)
            .filter_map(|query| match query {
                Query::Done(query) => Some(query.index()),
                Query::Add(_) | Query::Search(_) => None,
            })
            .collect::<Vec<_>>();

        self.todos
            .iter()
            .take_while(|(idx, _)| *idx < index)
            .filter_map(|(_, item)| {
                if completed.contains(&&item.index) {
                    Some(item)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl fmt::Display for Queries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = self.input().join("\n");
        write!(f, "{}", out)
    }
}

struct QueryAdd {
    description: todo::Description,
    tags: Vec<todo::Tag>,
}

impl QueryAdd {
    fn new(description: impl AsRef<str>) -> Self {
        Self {
            description: todo::Description::new(description.as_ref()),
            tags: vec![],
        }
    }

    fn with_tags(description: impl AsRef<str>, tags: Vec<impl AsRef<str>>) -> Self {
        Self {
            description: todo::Description::new(description.as_ref()),
            tags: tags
                .into_iter()
                .map(|tag| todo::Tag::new(tag.as_ref()))
                .collect(),
        }
    }

    fn random() -> Self {
        let desc: Vec<String> = Words(1..10).fake();
        let desc = desc.join(" ");

        let tags: Vec<String> = Words(0..10).fake();
        let tags = tags
            .into_iter()
            .map(|tag| todo::Tag::new(&tag))
            .collect::<Vec<_>>();

        Self {
            description: todo::Description::new(&desc),
            tags,
        }
    }

    /// Create a new add query with the given tags included. Other tags may also be added.
    /// # Arguments
    /// `tags`: List of tags to include. `#` should **NOT** be present.
    fn random_with_tags(mut tags: Vec<todo::Tag>) -> Self {
        let desc: Vec<String> = Words(1..10).fake();
        let desc = desc.join(" ");

        let new_tags: Vec<String> = Words(0..5).fake();
        let new_tags = new_tags
            .into_iter()
            .map(|tag| todo::Tag::new(&tag))
            .collect::<Vec<_>>();

        tags.extend(new_tags);

        Self {
            description: todo::Description::new(&desc),
            tags,
        }
    }

    /// # Returns
    /// A search word that matches the associated item's description.
    fn search_word(&self, rng: &mut impl Rng) -> String {
        let words = self.description.value().split(" ");
        let word = words.choose(rng).unwrap();

        let mut filter = Vec::with_capacity(word.chars().count());
        while filter.iter().filter(|x| **x).count() == 0 {
            // ensure filter is not empty
            filter = (0..word.chars().count())
                .map(|_| rng.gen_bool(0.5))
                .collect::<Vec<_>>();
        }

        word.chars()
            .enumerate()
            .filter_map(|(idx, c)| if filter[idx] { Some(c) } else { None })
            .collect()
    }

    fn random_tag(&self, rng: &mut impl Rng) -> Option<&todo::Tag> {
        if self.tags.is_empty() {
            None
        } else {
            Some(&self.tags[rng.gen_range(0, self.tags.len())])
        }
    }
}

impl fmt::Display for QueryAdd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tags = self
            .tags
            .iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        if tags.is_empty() {
            write!(f, r#"add "{}""#, self.description)
        } else {
            write!(f, r#"add "{}" {}"#, self.description, tags)
        }
    }
}

struct QueryDone(todo::Index);
impl QueryDone {
    pub fn new(index: u64) -> Self {
        Self(todo::Index::new(index))
    }

    pub fn index(&self) -> &todo::Index {
        &self.0
    }
}

impl fmt::Display for QueryDone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "done {}", self.0)
    }
}

#[derive(Clone)]
struct QuerySearch {
    words: Vec<String>,
    tags: Vec<todo::Tag>,
}

impl QuerySearch {
    pub fn new(words: Vec<impl Into<String>>, tags: Vec<impl AsRef<str>>) -> Self {
        Self {
            words: words.into_iter().map(|word| word.into()).collect(),
            tags: tags
                .into_iter()
                .map(|tag| todo::Tag::new(tag.as_ref()))
                .collect(),
        }
    }

    pub fn words(words: Vec<impl Into<String>>) -> Self {
        Self {
            words: words.into_iter().map(|word| word.into()).collect(),
            tags: vec![],
        }
    }

    pub fn tags(tags: Vec<impl AsRef<str>>) -> Self {
        Self {
            words: vec![],
            tags: tags
                .into_iter()
                .map(|tag| todo::Tag::new(tag.as_ref()))
                .collect(),
        }
    }
}

impl fmt::Display for QuerySearch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let words = self.words.join(" ");
        let tags = self
            .tags
            .iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "search {} {}", words, tags)
    }
}

/// # Returns
/// Whether the haystack contains the target as a substring.
fn contains_word(target: impl AsRef<str>, haystack: impl AsRef<str>) -> bool {
    haystack.as_ref().split(" ").any(|word| {
        let mut target_chars = target.as_ref().chars().peekable();
        let mut word_chars = word.chars().peekable();
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
    })
}
