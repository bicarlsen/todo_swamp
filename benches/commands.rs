use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{distributions::Standard, prelude::*};
use todo_swamp::*;

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

const INPUT_SIZES: &'static [usize] = &[500, 1_000, 1_500, 2_000, 2_500, 3_000, 4_000, 5_000];

pub fn commands(c: &mut Criterion) {
    const VOCABULARY_SIZE: usize = 10_000;

    let mut group = c.benchmark_group("commands");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));
    for n_cmds in INPUT_SIZES {
        group.throughput(criterion::Throughput::Elements(*n_cmds as u64));

        let mut rng = rand::thread_rng();
        let dictionary = Dictionary::new(&mut rng, VOCABULARY_SIZE);

        group.bench_with_input(BenchmarkId::from_parameter(n_cmds), &n_cmds, |bench, _| {
            bench.iter_batched(
                || {
                    let mut cmds = Vec::with_capacity(*n_cmds);
                    let init_add = rng.gen_range(5, 100);
                    (0..init_add).for_each(|_| cmds.push(Command::Add));
                    (0..n_cmds - init_add).for_each(|_| cmds.push(rng.gen()));

                    let mut word_vocabulary = Vec::with_capacity(*n_cmds);
                    let mut tag_vocabulary = Vec::with_capacity(*n_cmds);
                    let mut queries = Vec::with_capacity(*n_cmds);
                    for (idx, cmd) in cmds.into_iter().enumerate() {
                        let query = match cmd {
                            Command::Add => {
                                let (desc, tags) = add_query(&mut rng, &dictionary);
                                let words = desc.value().split(" ").map(|word| word.to_string());
                                word_vocabulary.extend(words);
                                tag_vocabulary.extend(tags.clone());
                                Query::Add((desc, tags))
                            }
                            Command::Done => {
                                let id = rng.gen_range(0, idx);
                                Query::Done(Index::new(id as u64))
                            }
                            Command::Search => {
                                let search =
                                    setup_search(&mut rng, &word_vocabulary, &tag_vocabulary);
                                Query::Search(search)
                            }
                        };

                        queries.push(query);
                    }

                    (TodoList::new(), queries)
                },
                |(mut todos, queries)| {
                    for query in queries {
                        match query {
                            Query::Add((desc, tags)) => {
                                todos.push(desc, tags);
                            }
                            Query::Done(index) => {
                                todos.done_with_index(index);
                            }
                            Query::Search(sp) => {
                                todos.search(sp);
                            }
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

pub fn commands_add(c: &mut Criterion) {
    const VOCABULARY_SIZE: usize = 10_000;

    let mut group = c.benchmark_group("commands_add");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));
    for n_cmds in INPUT_SIZES {
        group.throughput(criterion::Throughput::Elements(*n_cmds as u64));

        let mut rng = rand::thread_rng();
        let dictionary = Dictionary::new(&mut rng, VOCABULARY_SIZE);

        group.bench_with_input(BenchmarkId::from_parameter(n_cmds), &n_cmds, |bench, _| {
            bench.iter_batched(
                || {
                    let queries = (0..*n_cmds)
                        .map(|_| {
                            let (desc, tags) = add_query(&mut rng, &dictionary);
                            Query::Add((desc, tags))
                        })
                        .collect::<Vec<_>>();

                    (TodoList::new(), queries)
                },
                |(mut todos, queries)| {
                    for query in queries {
                        match query {
                            Query::Add((desc, tags)) => {
                                todos.push(desc, tags);
                            }
                            Query::Done(index) => {
                                todos.done_with_index(index);
                            }
                            Query::Search(sp) => {
                                todos.search(sp);
                            }
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
}

pub fn commands_done(c: &mut Criterion) {
    const VOCABULARY_SIZE: usize = 10_000;

    let mut group = c.benchmark_group("commands_done");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));
    for n_cmds in INPUT_SIZES {
        group.throughput(criterion::Throughput::Elements(*n_cmds as u64));

        let mut rng = rand::thread_rng();
        let dictionary = Dictionary::new(&mut rng, VOCABULARY_SIZE);

        group.bench_with_input(BenchmarkId::from_parameter(n_cmds), &n_cmds, |bench, _| {
            bench.iter_batched(
                || {
                    let mut todos = TodoList::with_capacity(*n_cmds);
                    let mut queries = Vec::with_capacity(*n_cmds);
                    for id in 0..*n_cmds {
                        let (desc, tags) = add_query(&mut rng, &dictionary);
                        todos.push(desc, tags);
                        queries.push(Query::Done(Index::new(id as u64)));
                    }

                    queries.shuffle(&mut rng);
                    (todos, queries)
                },
                |(mut todos, queries)| {
                    for query in queries {
                        match query {
                            Query::Add((desc, tags)) => {
                                todos.push(desc, tags);
                            }
                            Query::Done(index) => {
                                todos.done_with_index(index);
                            }
                            Query::Search(sp) => {
                                todos.search(sp);
                            }
                        }
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
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
    Add((Description, Vec<Tag>)),
    Done(Index),
    Search(SearchParams),
}

/// Create a random search.
/// The number of words and tags is random.
/// The number of matching words and tags is random.
fn setup_search(
    rng: &mut impl Rng,
    word_vocabulary: &Vec<String>,
    tag_vocabulary: &Vec<Tag>,
) -> SearchParams {
    let size_words = rng.gen_range(0, MAX_SEARCH_WORDS);
    let min_tags = if size_words == 0 { 1 } else { 0 };
    let size_tags = rng.gen_range(min_tags, MAX_SEARCH_TAGS);

    let words = setup_search_words(rng, size_words, word_vocabulary);
    let tags = setup_search_tags(rng, size_tags, tag_vocabulary);
    SearchParams { words, tags }
}

/// Create a random list of words to search for.
/// Some words may match the vocabulary, and some may not.
fn setup_search_words(
    rng: &mut impl Rng,
    size: usize,
    vocabulary: &Vec<String>,
) -> Vec<SearchWord> {
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
fn setup_search_tags(rng: &mut impl Rng, size: usize, vocabulary: &Vec<Tag>) -> Vec<Tag> {
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
) -> Vec<SearchWord> {
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
        .map(|word| SearchWord(word))
        .collect()
}

/// Create a random list of words to search for.
/// Words are guaranteed not to match any in the vocabulary.
fn setup_search_words_not_matching(rng: &mut impl Rng, size: usize) -> Vec<SearchWord> {
    (0..size)
        .map(|_| {
            let prefix = gen_word(rng);
            let postfix = gen_word(rng);
            SearchWord(format!("{prefix}0{postfix}"))
        })
        .collect()
}

/// Create a random list of tags to search for.
/// Tags are guaranteed to match the vocabulary.
fn setup_search_tags_matching(rng: &mut impl Rng, size: usize, vocabulary: &Vec<Tag>) -> Vec<Tag> {
    (0..size)
        .map(|_| vocabulary.choose(rng).unwrap().clone())
        .collect()
}

/// Create a random list of tags to search for.
/// Tags are guaranteed not to match any in the vocabulary.
fn setup_search_tags_not_matching(rng: &mut impl Rng, size: usize) -> Vec<Tag> {
    (0..size)
        .map(|_| {
            let prefix = gen_word(rng);
            let postfix = gen_word(rng);
            let tag = format!("{prefix}0{postfix}");
            Tag::new(&tag)
        })
        .collect()
}

/// # Returns
/// A search word that matches the associated item's description.
fn search_word(description: &Description, rng: &mut impl Rng) -> String {
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

fn add_query(rng: &mut impl Rng, dictionary: &Dictionary) -> (Description, Vec<Tag>) {
    let desc = dictionary
        .words(rng, 1, MAX_ITEM_DESCRIPTION_WORDS)
        .join(" ");

    let tags = dictionary
        .words(rng, 0, MAX_ITEM_TAGS)
        .into_iter()
        .map(|tag| Tag::new(&tag))
        .collect::<Vec<_>>();

    (Description::new(&desc), tags)
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

criterion_group!(bench_commands, commands);
criterion_main!(bench_commands);
