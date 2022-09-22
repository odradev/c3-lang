use std::fmt::Debug;

use c3_lang_linearization::{Class, Fn};
use proc_macro2::Ident;
use syn::{Attribute, Block, FnArg, Item, ReturnType, Type};

#[derive(Debug, PartialEq)]
pub struct PackageDef {
    pub other_code: Vec<Item>,
    pub class_name: ClassNameDef,
    pub classes: Vec<ClassDef>,
}

#[derive(Debug, PartialEq)]
pub struct ClassNameDef {
    pub classes: Vec<Class>,
}

#[derive(Debug, PartialEq)]
pub struct ClassDef {
    pub struct_attrs: Vec<Attribute>,
    pub impl_attrs: Vec<Attribute>,
    pub class: Class,
    pub path: Vec<Class>,
    pub variables: Vec<VarDef>,
    pub functions: Vec<FnDef>,
}

#[derive(Debug, PartialEq)]
pub struct VarDef {
    pub ident: Ident,
    pub ty: Type,
}

#[derive(Debug, PartialEq)]
pub struct FnDef {
    pub attrs: Vec<Attribute>,
    pub name: Fn,
    pub args: Vec<FnArg>,
    pub ret: ReturnType,
    pub implementations: Vec<ClassFnImpl>,
}

#[derive(Debug, PartialEq)]
pub struct ClassFnImpl {
    pub class: Class,
    pub fun: Fn,
    pub implementation: Block,
}
