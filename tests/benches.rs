#![feature(test)]
extern crate test;

use rand::{distributions::Standard, prelude::*};
use std::fmt;
use todo_swamp as todo;

/// Maximum length of a word.
const MAX_WORD_LENGTH: usize = 10;

/// Max number of words in an item's description.
const MAX_ITEM_DESCRIPTION_WORDS: usize = 30;

/// Max number of tags per item.
const MAX_ITEM_TAGS: usize = 10;

/// Max number of tags in a search.
const MAX_SEARCH_WORDS: usize = 10;

/// Max number of tags in a search.
const MAX_SEARCH_TAGS: usize = 10;

#[bench]
fn small(bench: &mut test::Bencher) {
    const VOCABULARY_SIZE: usize = 10_000;
    const NUM_ITEMS: usize = 100;

    let mut rng = rand::thread_rng();
    let dictionary = Dictionary::new(&mut rng, VOCABULARY_SIZE);
    let adds = (0..NUM_ITEMS)
        .map(|_| {
            let (words, tags) = add_query(&mut rng, &dictionary);
            QueryAdd::from(words, tags)
        })
        .collect::<Vec<_>>();

    let mut todos = todo::TodoList::new();
    for query in adds {
        todos.push(query.description, query.tags);
    }

    bench.iter(|| {});
}

enum Command {
    Add,
    Done,
    Search,
}

impl Distribution<Command> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Command {
        match rng.gen_range(0, 3) {
            0 => Command::Add,
            1 => Command::Done,
            2 => Command::Search,
            _ => unreachable!(),
        }
    }
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

    fn random(rng: &mut impl Rng, dictionary: &mut Dictionary, size: usize) -> Self {
        let mut cmds = Vec::with_capacity(size);
        let init_add = rng.gen_range(5, 100);
        (0..init_add).for_each(|_| cmds.push(Command::Add));
        (0..size - init_add).for_each(|_| cmds.push(rng.gen()));

        let mut word_vocabulary = Vec::with_capacity(size as usize);
        let mut tag_vocabulary = Vec::with_capacity(size as usize);
        let mut queries = Vec::with_capacity(size as usize);
        let mut idx = 0;
        for cmd in cmds.into_iter() {
            let query = match cmd {
                Command::Add => {
                    let (desc, tags) = add_query(rng, dictionary);
                    let words = desc.value().split(" ").map(|word| word.to_string());
                    word_vocabulary.extend(words);
                    tag_vocabulary.extend(tags.clone());
                    idx += 1;
                    Query::Add(QueryAdd::from(desc, tags))
                }
                Command::Done => {
                    let id = rng.gen_range(0, idx);
                    Query::Done(QueryDone(todo::Index::new(id as u64)))
                }
                Command::Search => {
                    let (words, tags) = setup_search(rng, &word_vocabulary, &tag_vocabulary);
                    Query::Search(QuerySearch::new(words, tags))
                }
            };

            queries.push(query);
        }

        Self::from(queries)
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
    pub fn from(description: todo::Description, tags: Vec<todo::Tag>) -> Self {
        Self { description, tags }
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
    pub fn new(words: Vec<impl Into<String>>, tags: Vec<todo::Tag>) -> Self {
        Self {
            words: words.into_iter().map(|word| word.into()).collect(),
            tags,
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

    pub fn as_search_params(&self) -> todo::SearchParams {
        let words = self
            .words
            .iter()
            .map(|word| todo::SearchWord(word.clone()))
            .collect();

        todo::SearchParams {
            words,
            tags: self.tags.clone(),
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

/// Create a random search.
/// The number of words and tags is random.
/// The number of matching words and tags is random.
fn setup_search(
    rng: &mut impl Rng,
    word_vocabulary: &Vec<String>,
    tag_vocabulary: &Vec<todo::Tag>,
) -> (Vec<String>, Vec<todo::Tag>) {
    let size_words = rng.gen_range(0, MAX_SEARCH_WORDS);
    let min_tags = if size_words == 0 { 1 } else { 0 };
    let size_tags = rng.gen_range(min_tags, MAX_SEARCH_TAGS);

    let words = setup_search_words(rng, size_words, word_vocabulary);
    let tags = setup_search_tags(rng, size_tags, tag_vocabulary);
    (words, tags)
}

/// Create a random list of words to search for.
/// Some words may match the vocabulary, and some may not.
fn setup_search_words(rng: &mut impl Rng, size: usize, vocabulary: &Vec<String>) -> Vec<String> {
    let mut words = Vec::with_capacity(size);
    let n_matching = rng.gen_range(0, size + 1);
    let matching = setup_search_words_matching(rng, n_matching, vocabulary);
    let not_matching = setup_search_words_not_matching(rng, size - n_matching);
    words.extend(matching);
    words.extend(not_matching);
    words.shuffle(rng);
    words
}

/// Create a random list of words to search for.
/// Some words may match the vocabulary, and some may not.
fn setup_search_tags(
    rng: &mut impl Rng,
    size: usize,
    vocabulary: &Vec<todo::Tag>,
) -> Vec<todo::Tag> {
    let mut tags = Vec::with_capacity(size);
    let n_matching = rng.gen_range(0, size + 1);
    let matching = setup_search_tags_matching(rng, n_matching, vocabulary);
    let not_matching = setup_search_tags_not_matching(rng, size - n_matching);
    tags.extend(matching);
    tags.extend(not_matching);
    tags.shuffle(rng);
    tags
}

/// Create a random list of words to search for.
/// Words are guaranteed to match the vocabulary.
fn setup_search_words_matching(
    rng: &mut impl Rng,
    size: usize,
    vocabulary: &Vec<String>,
) -> Vec<String> {
    /// Likelihood to modify a word.
    const P_MODIFY_WORD: f64 = 0.9;

    /// Likelihood to remove a letter.
    const P_REMOVE_LETTER: f64 = 0.3;

    (0..size)
        .map(|_| {
            let word = vocabulary.choose(rng).unwrap().clone();
            if word.len() == 1 {
                return word;
            }
            if !rng.gen_bool(P_MODIFY_WORD) {
                return word;
            }

            let mut modified = String::new();
            while modified.is_empty() {
                modified = word
                    .chars()
                    .filter(|_| rng.gen_bool(P_REMOVE_LETTER))
                    .collect();
            }
            modified
        })
        .collect()
}

/// Create a random list of words to search for.
/// Words are guaranteed not to match any in the vocabulary.
fn setup_search_words_not_matching(rng: &mut impl Rng, size: usize) -> Vec<String> {
    (0..size)
        .map(|_| {
            let prefix = gen_word(rng);
            let postfix = gen_word(rng);
            format!("{prefix}0{postfix}")
        })
        .collect()
}

/// Create a random list of tags to search for.
/// Tags are guaranteed to match the vocabulary.
fn setup_search_tags_matching(
    rng: &mut impl Rng,
    size: usize,
    vocabulary: &Vec<todo::Tag>,
) -> Vec<todo::Tag> {
    (0..size)
        .map(|_| vocabulary.choose(rng).unwrap().clone())
        .collect()
}

/// Create a random list of tags to search for.
/// Tags are guaranteed not to match any in the vocabulary.
fn setup_search_tags_not_matching(rng: &mut impl Rng, size: usize) -> Vec<todo::Tag> {
    (0..size)
        .map(|_| {
            let prefix = gen_word(rng);
            let postfix = gen_word(rng);
            let tag = format!("{prefix}0{postfix}");
            todo::Tag::new(&tag)
        })
        .collect()
}

/// # Returns
/// A search word that matches the associated item's description.
fn search_word(description: &todo::Description, rng: &mut impl Rng) -> String {
    let words = description.value().split(" ");
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

fn add_query(rng: &mut impl Rng, dictionary: &Dictionary) -> (todo::Description, Vec<todo::Tag>) {
    let desc = dictionary
        .words(rng, 1, MAX_ITEM_DESCRIPTION_WORDS)
        .join(" ");

    let tags = dictionary
        .words(rng, 0, MAX_ITEM_TAGS)
        .into_iter()
        .map(|tag| todo::Tag::new(&tag))
        .collect::<Vec<_>>();

    (todo::Description::new(&desc), tags)
}

struct Dictionary {
    vocabulary: Vec<String>,
}

impl Dictionary {
    pub const ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz-";

    pub fn new(rng: &mut impl Rng, size: usize) -> Self {
        let mut vocabulary = (0..size).map(|_| gen_word(rng)).collect::<Vec<_>>();
        vocabulary.sort();
        Self { vocabulary }
    }

    pub fn words(&self, rng: &mut impl Rng, low: usize, high: usize) -> Vec<String> {
        let n = rng.gen_range(low, high);
        (0..n)
            .map(|_| self.vocabulary.choose(rng).unwrap().clone())
            .collect()
    }
}

pub fn gen_word(rng: &mut impl Rng) -> String {
    let n = rng.gen_range(1, MAX_WORD_LENGTH);
    (0..n)
        .map(|_| Dictionary::ALPHABET.chars().choose(rng).unwrap())
        .collect()
}

pub fn gen_words(rng: &mut impl Rng, low: usize, high: usize) -> Vec<String> {
    let n = rng.gen_range(low, high);
    (0..n).map(|_| gen_word(rng)).collect()
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
