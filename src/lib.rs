#![allow(dead_code)]
pub mod commands;
pub mod database;
pub mod media;
pub mod query;
pub mod sync_db;
pub mod vault;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
