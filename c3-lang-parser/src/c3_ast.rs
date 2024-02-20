use std::fmt::Debug;

use c3_lang_linearization::{Class, Fn};
use proc_macro2::Ident;
use syn::{Attribute, Block, FnArg, Item, ReturnType, Type, Visibility};

#[derive(Debug, PartialEq)]
pub struct PackageDef {
    pub no_std: bool,
    pub attrs: Vec<Attribute>,
    pub other_code: Vec<Item>,
    pub class_name: ClassNameDef,
    pub classes: Vec<ClassDef>,
}

#[derive(Debug, PartialEq)]
pub struct ClassNameDef {
    pub classes: Vec<Class>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassDef {
    pub struct_attrs: Vec<Attribute>,
    pub impl_attrs: Vec<Attribute>,
    pub class: Class,
    pub path: Vec<Class>,
    pub variables: Vec<VarDef>,
    pub other_items: Vec<Item>,
    pub functions: Vec<FnDef>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDef {
    pub ident: Ident,
    pub ty: Type,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FnDef {
    Plain(PlainFnDef),
    Complex(ComplexFnDef),
}

impl FnDef {
    pub fn has_receiver_arg(&self) -> bool {
        match self {
            FnDef::Plain(f) => f.args.len() > 0 && matches!(f.args[0], FnArg::Receiver(_)),
            FnDef::Complex(f) => f.args.len() > 0 && matches!(f.args[0], FnArg::Receiver(_)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PlainFnDef {
    pub attrs: Vec<Attribute>,
    pub name: Fn,
    pub args: Vec<FnArg>,
    pub ret: ReturnType,
    pub implementation: ClassFnImpl,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ComplexFnDef {
    pub attrs: Vec<Attribute>,
    pub name: Fn,
    pub args: Vec<FnArg>,
    pub ret: ReturnType,
    pub implementations: Vec<ClassFnImpl>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassFnImpl {
    pub visibility: Visibility,
    pub class: Option<Class>,
    pub fun: Fn,
    pub implementation: Block,
}
