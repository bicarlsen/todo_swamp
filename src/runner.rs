use crate::*;

pub fn run_line(line: &str, tl: &mut TodoList) {
    if let Ok((_, q)) = parser::query(line) {
        match run_query(q, tl) {
            Ok(r) => {
                println!("{}", r);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
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
