use std::collections::HashMap;

use crate::{id::Id, split_coma};

use super::{C3Error, Sets};

pub type Class = Id;
pub type Fn = Id;
pub type Var = Id;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct C3 {
    /// HashMap that maps base classes to path list.
    classes: HashMap<Class, Vec<Class>>,
    functions: HashMap<Class, Vec<Fn>>,
    variables: HashMap<Class, Vec<Var>>,
}

impl C3 {
    /// Build new empty instance.
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// Add new class via strings.
    pub fn add(&mut self, base: Class, path: Vec<Class>) {
        self.classes.insert(base, path);
    }

    /// Add new class via strings.
    pub fn add_class_str(&mut self, base: &str, parents: &str) {
        let input: Vec<Class> = split_coma(parents).into_iter().map(Class::from).collect();
        self.add(Class::from(base), input);
    }

    /// Remove an element. Fails if element doesn't exists.
    pub fn remove(&mut self, base: &Class) -> Result<(), C3Error> {
        match self.classes.remove(base) {
            Some(_) => Ok(()),
            None => Err(C3Error::BaseClassDoesNotExists(base.clone().into())),
        }
    }

    /// Returns all base classes as a vector Classes.
    pub fn all_classes(&self) -> Vec<Class> {
        let mut keys: Vec<Class> = self.classes.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Returns all base classes as a vector of Strings.
    pub fn all_classes_str(&self) -> Vec<String> {
        self.all_classes()
            .into_iter()
            .map(|class| class.into())
            .collect()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    /// Return list of parents for a given base class.
    pub fn path(&self, base: &Class) -> Result<Vec<Class>, C3Error> {
        match self.classes.get(base) {
            Some(path) => Ok(path.clone()),
            None => Err(C3Error::BaseClassDoesNotExists(base.clone().into())),
        }
    }

    /// Prepare sets for the merge function.
    pub fn sets_for(&self, bases: Vec<Class>) -> Result<Sets<Class>, C3Error> {
        let mut sets = Sets::new();
        for base in &bases {
            let path = self.path(base)?;
            if !path.is_empty() {
                sets.push(path.clone())?;
            }
        }
        if !bases.is_empty() {
            sets.push(bases)?;
        }
        Ok(sets)
    }

    pub fn register_fn(&mut self, class: Class, fun: Fn) {
        self.functions.entry(class).or_insert(vec![]).push(fun);
    }

    pub fn register_fns(&mut self, class: Class, funs: Vec<Fn>) {
        for fun in funs {
            self.register_fn(class.clone(), fun);
        }
    }

    pub fn register_fn_str(&mut self, class: &str, function: &str) {
        self.register_fn(Class::from(class), Fn::from(function))
    }

    pub fn functions(&self, class: &Class) -> Vec<Fn> {
        let path: Vec<Class> = self.path(class).unwrap();
        let mut functions: Vec<Fn> = vec![];
        for class in path {
            let mut list: Vec<Fn> = self.functions.get(&class).cloned().unwrap_or_default();
            functions.append(&mut list);
        }
        functions.sort();
        functions.dedup();
        functions
    }

    pub fn functions_str(&self, class: &str) -> Vec<String> {
        self.functions(&Class::from(class))
            .into_iter()
            .map(|x| x.to_string())
            .collect()
    }

    pub fn register_var(&mut self, class: Class, var: Var) {
        self.variables.entry(class).or_insert(vec![]).push(var);
    }

    pub fn register_vars(&mut self, class: Class, vars: Vec<Var>) {
        for var in vars {
            self.register_var(class.clone(), var);
        }
    }

    pub fn register_var_str(&mut self, class: &str, variable: &str) {
        self.register_var(Class::from(class), Var::from(variable))
    }

    pub fn variables(&self, class: &Class) -> Vec<Var> {
        let path: Vec<Class> = self.path(class).unwrap();
        let mut varialbes: Vec<Var> = vec![];
        for class in path {
            let mut list: Vec<Var> = self.variables.get(&class).cloned().unwrap_or_default();
            varialbes.append(&mut list);
        }
        varialbes.sort();
        varialbes.dedup();
        varialbes
    }

    pub fn varialbes_str(&self, class: &str) -> Vec<String> {
        self.variables(&Class::from(class))
            .into_iter()
            .map(|x| x.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::C3;

    #[test]
    fn test_c3() {
        let mut c3 = C3::new();
        c3.add_class_str("A", "A");
        c3.add_class_str("B", "B, A");
        c3.add_class_str("K1", "K1, K2, A");

        assert_eq!(c3.all_classes_str(), vec!["A", "B", "K1"]);

        c3.register_fn_str("A", "foo");
        c3.register_fn_str("A", "bar");
        c3.register_fn_str("B", "bar");
        assert_eq!(c3.functions_str("A"), vec!["bar", "foo"]);
        assert_eq!(c3.functions_str("B"), vec!["bar", "foo"]);

        c3.register_var_str("A", "x");
        c3.register_var_str("B", "y");
        assert_eq!(c3.varialbes_str("A"), vec!["x"]);
        assert_eq!(c3.varialbes_str("B"), vec!["x", "y"]);
    }

    // TODO: More tests.
}
