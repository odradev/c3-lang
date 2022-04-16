use super::{in_tail, C3Error};

// TODO: Comment and test.
pub struct Sets<T> {
    sets: Vec<Vec<T>>,
}

impl<T: Clone + Eq + PartialEq> Sets<T> {
    pub fn new() -> Self {
        Self { sets: Vec::new() }
    }

    pub fn push(&mut self, set: Vec<T>) -> Result<(), C3Error> {
        if set.is_empty() {
            Err(C3Error::PushingEmptySet)
        } else {
            self.sets.push(set);
            Ok(())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sets.is_empty()
    }

    pub fn find_solution(&mut self) -> Result<T, C3Error> {
        for candidate in self.candidates() {
            if self.is_solution(&candidate)? {
                self.remove_solution(&candidate);
                return Ok(candidate);
            }
        }
        Err(C3Error::NoMoreCandidates)
    }

    fn candidates(&self) -> Vec<T> {
        self.sets
            .iter()
            .map(|set| set.first().unwrap())
            .cloned()
            .collect()
    }

    fn is_solution(&self, candidate: &T) -> Result<bool, C3Error> {
        for set in &self.sets {
            if in_tail(candidate, set)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn remove_solution(&mut self, solution: &T) {
        for set in self.sets.iter_mut() {
            set.retain(|x| x != solution);
        }
        self.sets.retain(|set| !set.is_empty());
    }
}
