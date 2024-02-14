use c3_lang_linearization::Class;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{parse_quote, Attribute, FnArg};

use super::c3_ast::{ClassDef, ClassFnImpl, ClassNameDef, FnDef, PackageDef, VarDef};

impl ToTokens for PackageDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(&self.attrs);
        tokens.append_all(&self.other_code);
        tokens.extend(stack_definition(self.classes[0].path.clone()));
        tokens.extend(self.class_name.to_token_stream());
        tokens.append_all(&self.classes);
    }
}

impl ToTokens for ClassNameDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let classes = &self.classes;
        tokens.extend(quote! {
            #[derive(Clone, Copy)]
            enum ClassName {
                #(#classes),*
            }
        })
    }
}

impl ToTokens for ClassDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let class_ident = &self.class;
        let variables = &self.variables;
        let functions = &self.functions;
        let struct_attrs = attributes_to_token_stream(&self.struct_attrs);
        let impl_attrs = attributes_to_token_stream(&self.impl_attrs);
        let other_items = &self.other_items;

        tokens.extend(quote! {
            #struct_attrs
            pub struct #class_ident {
                #(#variables),*
            }

            #impl_attrs
            impl #class_ident {
                #(#other_items)*

                #(#functions)*
            }
        })
    }
}

impl ToTokens for VarDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let ty = &self.ty;
        tokens.extend(quote! {
            #ident: #ty
        });
    }
}

impl ToTokens for FnDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FnDef::Plain(def) => {
                let fn_ident = &def.name;
                let args = &def.args;
                let ret = &def.ret;
                let implementation = &def.implementation;
                let attrs = attributes_to_token_stream(&def.attrs);
                let vis = &implementation.visibility;

                tokens.extend(quote! {
                    #attrs
                    #vis fn #fn_ident(#(#args),*) #ret
                        #implementation
                });
            }
            FnDef::Complex(def) => {
                let fn_ident = &def.name;
                let fn_super_ident = format_ident!("super_{}", fn_ident.to_string());
                let args = &def.args;
                let ret = &def.ret;
                let implementations = &def.implementations;
                let args_as_params = args_to_params(args);
                let attrs = attributes_to_token_stream(&def.attrs);
                let vis = implementations
                    .first()
                    .map(|f| f.visibility.clone())
                    .unwrap_or(parse_quote!(pub));

                tokens.extend(quote! {
                    #attrs
                    #vis fn #fn_ident(#(#args),*) #ret {
                        unsafe { STACK.push_path_on_stack(); }
                        let result = self.#fn_super_ident(#(#args_as_params),*);
                        unsafe { STACK.drop_one_from_stack(); }
                        result
                    }

                    fn #fn_super_ident(#(#args),*) #ret {
                        let __class = unsafe { STACK.pop_from_top_path() };
                        match __class {
                            #(#implementations),*
                            #[allow(unreachable_patterns)]
                            _ => self.#fn_super_ident(#(#args_as_params),*),
                        }
                    }
                });
            }
        };
    }
}

impl ToTokens for ClassFnImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let class = self
            .class
            .as_ref()
            .map(|class| quote!(Some(ClassName::#class) => ));
        let implementation = &self.implementation;
        tokens.extend(quote! {
            #class #implementation
        });
    }
}

fn args_to_params(args: &[FnArg]) -> Vec<Ident> {
    args.iter()
        .skip(1)
        .map(|arg| {
            if let FnArg::Typed(arg) = arg {
                format_ident!("{}", arg.pat.to_token_stream().to_string())
            } else {
                panic!("Unsupported arg");
            }
        })
        .collect()
}

fn attributes_to_token_stream(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    let mut result = proc_macro2::TokenStream::new();
    for attr in attrs {
        result.extend(quote! {
            #attr
        });
    }
    result
}

fn stack_definition(path: Vec<Class>) -> TokenStream {
    let path: Vec<Class> = path.clone().into_iter().rev().collect();
    let path_len = path.len();
    quote! {
        #[derive(Clone)]
        struct PathStack {
            path: [ClassName; MAX_PATH_LENGTH],
            stack_pointer: usize,
            path_pointer: usize,
        }

        impl PathStack {
            pub fn push_path_on_stack(&mut self) {
                self.path_pointer = 0;
                if self.stack_pointer < MAX_STACK_SIZE {
                    self.stack_pointer += 1;
                }
            }

            pub fn drop_one_from_stack(&mut self) {
                if self.stack_pointer > 0 {
                    self.stack_pointer -= 1;
                }
            }

            pub fn pop_from_top_path(&mut self) -> Option<ClassName> {
                if self.path_pointer < MAX_PATH_LENGTH {
                    let class = self.path[MAX_PATH_LENGTH - self.path_pointer - 1];
                    self.path_pointer += 1;
                    Some(class)
                } else {
                    None
                }
            }
        }

        static mut STACK: PathStack = PathStack::new();
        const MAX_STACK_SIZE: usize = 8; // Maximum number of paths in the stack
        const MAX_PATH_LENGTH: usize = #path_len; // Maximum length of each path
        impl PathStack {
            pub const fn new() -> Self {
                Self {
                    path: [#(ClassName::#path),*],
                    stack_pointer: 0,
                    path_pointer: 0,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use c3_lang_linearization::Class;
    use quote::{quote, ToTokens};

    use crate::{c3_ast_builder::tests::test_c3_ast, test_utils::test_code};

    use super::stack_definition;

    #[test]
    fn test_package_def_printing() {
        let input = test_c3_ast();
        let stack = stack_definition(vec![Class::from("B"), Class::from("A")]);
        let target = quote! {
            pub type Num = u32;

            #stack

            #[derive(Clone)]
            enum ClassName {
                A,
                B,
            }

            #[derive(Debug)]
            pub struct B {
                x: u32,
            }

            #[cfg(target_os = "linux")]
            impl B {
                pub fn bar(&self, counter: Num) -> String {
                    unsafe { STACK.push_path_on_stack(); }
                    let result = self.super_bar(counter);
                    unsafe { STACK.drop_one_from_stack(); }
                    result
                }
                fn super_bar(&self, counter: Num) -> String {
                    let __class = unsafe { STACK.pop_from_top_path() };
                    match __class {
                        Some(ClassName::A) => {
                            let label = format!("A::bar({})", counter);
                            if counter == 0 {
                                label
                            } else {
                                format!("{} {}", label, self.foo(counter - 1))
                            }
                        }
                        Some(ClassName::B) => {
                            let label = format!("B::bar({})", counter);
                            if counter == 0 {
                                label
                            } else {
                                format!("{} {}", label, self.super_bar(counter - 1))
                            }
                        }
                        #[allow(unreachable_patterns)]
                        _ => self.super_bar(counter),
                    }
                }

                #[test]
                pub fn foo(&self, counter: Num) -> String {
                    unsafe { STACK.push_path_on_stack(); }
                    let result = self.super_foo(counter);
                    unsafe { STACK.drop_one_from_stack(); }
                    result
                }
                fn super_foo(&self, counter: Num) -> String {
                    let __class = unsafe { STACK.pop_from_top_path() };
                    match __class {
                        Some(ClassName::A) => {
                            let label = format!("A::foo({})", counter);
                            if counter == 0 {
                                label
                            } else {
                                format!("{} {}", label, self.bar(counter - 1))
                            }
                        }
                        #[allow(unreachable_patterns)]
                        _ => self.super_foo(counter),
                    }
                }
            }

        };
        test_code(input.to_token_stream(), target)
    }
}
