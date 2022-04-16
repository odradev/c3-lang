mod c3;
mod c3_linearization;
mod id;
mod sets;

pub use crate::c3::{Class, Fn, Var, C3};
pub use c3_linearization::c3_linearization;
use sets::Sets;

#[derive(Debug)]
pub enum C3Error {
    BaseClassDoesNotExists(String),
    EmptySet,
    PushingEmptySet,
    NoMoreCandidates,
}

/// Parse coma separated list of parents.
/// TODO: Implement using regexp
pub fn split_coma(s: &str) -> Vec<String> {
    s.split(", ")
        .map(String::from)
        .filter(|x| !x.is_empty())
        .collect()
}

/// Check if element is in the tail of the list.
fn in_tail<T: Eq>(element: &T, list: &[T]) -> Result<bool, C3Error> {
    match list.split_first() {
        Some((_, tail)) => Ok(tail.contains(element)),
        None => Err(C3Error::EmptySet),
    }
}

/// Check if is `smaller` is subset of `larger`.
pub fn is_subset<T: Eq>(larger: &[T], smaller: &[T]) -> bool {
    smaller.iter().all(|item| larger.contains(item))
}
