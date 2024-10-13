extern crate todo_swamp;

use std::io;
use std::io::prelude::*;

use todo_swamp::*;

pub fn main() {
    let mut tl: TodoList = TodoList::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(l) = line {
            runner::run_line(&l, &mut tl);
        }
    }
}
