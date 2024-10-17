use crate::*;
use std::io;

pub fn run_line(line: &str, tl: &mut TodoList, out: &mut impl io::Write, err: &mut impl io::Write) {
    if let Ok((_, q)) = parser::query(line) {
        match run_query(q, tl) {
            Ok(r) => {
                writeln!(out, "{}", r).expect("could not write to out");
            }
            Err(e) => {
                writeln!(err, "Error: {}", e).expect("could not write to err");
            }
        }
    }
}

fn run_query(q: Query, tl: &mut TodoList) -> Result<QueryResult, QueryError> {
    match q {
        Query::Add(desc, tags) => {
            let item = tl.push(desc, tags);
            Ok(QueryResult::Added(item))
        }
        Query::Done(idx) => tl
            .done_with_index(idx)
            .map(|_idx| QueryResult::Done)
            .ok_or(QueryError("item does not exist".to_string())),
        Query::Search(params) => Ok(QueryResult::Found(
            tl.search(params).into_iter().cloned().collect(),
        )),
    }
}
