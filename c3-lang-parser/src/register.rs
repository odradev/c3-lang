use std::collections::HashMap;

use c3_lang_linearization::{Class, Fn, Var};
use syn::{Field, ImplItemMethod};

#[derive(Default)]
pub struct Register {
    functions: HashMap<Fn, Vec<(Class, ImplItemMethod)>>,
    variables: HashMap<Var, Field>,
}

impl Register {
    pub fn add(&mut self, class: Class, fun: Fn, fun_impl: ImplItemMethod) {
        self.functions
            .entry(fun)
            .or_default()
            .push((class, fun_impl));
    }

    pub fn get_first_impl(&self, fun: &Fn) -> ImplItemMethod {
        self.get(fun).first().unwrap().clone().1
    }

    pub fn get(&self, fun: &Fn) -> Vec<(Class, ImplItemMethod)> {
        self.functions.get(fun).unwrap().clone()
    }

    pub fn functions(&self) -> Vec<Fn> {
        let mut list: Vec<Fn> = self.functions.keys().cloned().collect();
        list.sort();
        list
    }

    pub fn add_var(&mut self, var: Var, field: Field) {
        self.variables.insert(var, field);
    }

    pub fn get_var(&self, var: Var) -> Field {
        self.variables.get(&var).unwrap().clone()
    }
}
