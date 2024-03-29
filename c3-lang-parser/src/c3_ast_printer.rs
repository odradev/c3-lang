use c3_lang_linearization::Class;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{Attribute, FnArg};

use super::c3_ast::{ClassDef, ClassFnImpl, ClassNameDef, FnDef, PackageDef, VarDef};

impl ToTokens for PackageDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(&self.other_code);
        tokens.extend(stack_definition());
        tokens.extend(self.class_name.to_token_stream());
        tokens.append_all(&self.classes);
    }
}

impl ToTokens for ClassNameDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let classes = &self.classes;
        tokens.extend(quote! {
            #[derive(Clone)]
            enum ClassName {
                #(#classes),*
            }
        })
    }
}

impl ToTokens for ClassDef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let class_ident = &self.class;
        let path: Vec<Class> = self.path.clone().into_iter().rev().collect();
        let path_len = path.len();
        let variables = &self.variables;
        let functions = &self.functions;
        let struct_attrs = attributes_to_token_stream(&self.struct_attrs);
        let impl_attrs = attributes_to_token_stream(&self.impl_attrs);
        tokens.extend(quote! {
            #struct_attrs
            pub struct #class_ident {
                __stack: PathStack,
                #(#variables),*
            }

            #impl_attrs
            impl #class_ident {
                const PATH: &'static [ClassName; #path_len] = &[
                    #(ClassName::#path),*
                ];

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
        let fn_ident = &self.name;
        let fn_super_ident = format_ident!("super_{}", fn_ident.to_string());
        let args = &self.args;
        let ret = &self.ret;
        let implementations = &self.implementations;
        let args_as_params = args_to_params(args);
        let attrs = attributes_to_token_stream(&self.attrs);
        tokens.extend(quote! {
            #attrs
            pub fn #fn_ident(#(#args),*) #ret {
                self.__stack.push_path_on_stack(Self::PATH);
                let result = self.#fn_super_ident(#(#args_as_params),*);
                self.__stack.drop_one_from_stack();
                result
            }

            pub fn #fn_super_ident(#(#args),*) #ret {
                let __class = self.__stack.pop_from_top_path();
                match __class {
                    #(#implementations),*
                    #[allow(unreachable_patterns)]
                    _ => self.#fn_super_ident(#(#args_as_params),*),
                }
            }
        });
    }
}

impl ToTokens for ClassFnImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let class = &self.class;
        let implementation = &self.implementation;
        tokens.extend(quote! {
            ClassName::#class => #implementation
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

fn stack_definition() -> TokenStream {
    quote! {
        struct PathStack {
            stack: std::sync::Arc<std::sync::Mutex<Vec<Vec<ClassName>>>>
        }

        impl PathStack {
            pub fn new() -> Self {
                PathStack {
                    stack: std::sync::Arc::new(std::sync::Mutex::new(Vec::new()))
                }
            }

            pub fn push_path_on_stack(&self, path: &[ClassName]) {
                let mut stack = self.stack.lock().unwrap();
                stack.push(path.to_vec());
            }

            pub fn drop_one_from_stack(&self) {
                let mut stack = self.stack.lock().unwrap();
                stack.pop().unwrap();
            }

            pub fn pop_from_top_path(&self) -> ClassName {
                let mut stack = self.stack.lock().unwrap();
                let mut path = stack.pop().unwrap();
                let class = path.pop().unwrap();
                stack.push(path);
                class
            }
        }

        impl Default for PathStack {
            fn default() -> PathStack {
                PathStack::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::{quote, ToTokens};

    use crate::{c3_ast_builder::tests::test_c3_ast, test_utils::test_code};

    use super::stack_definition;

    #[test]
    fn test_package_def_printing() {
        let input = test_c3_ast();
        let stack = stack_definition();
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
                __stack: PathStack,
                x: u32,
            }

            #[cfg(target_os = "linux")]
            impl B {
                const PATH: &'static [ClassName; 2usize] = &[ClassName::A, ClassName::B];

                pub fn bar(&self, counter: Num) -> String {
                    self.__stack.push_path_on_stack(Self::PATH);
                    let result = self.super_bar(counter);
                    self.__stack.drop_one_from_stack();
                    result
                }
                pub fn super_bar(&self, counter: Num) -> String {
                    let __class = self.__stack.pop_from_top_path();
                    match __class {
                        ClassName::A => {
                            let label = format!("A::bar({})", counter);
                            if counter == 0 {
                                label
                            } else {
                                format!("{} {}", label, self.foo(counter - 1))
                            }
                        }
                        ClassName::B => {
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
                    self.__stack.push_path_on_stack(Self::PATH);
                    let result = self.super_foo(counter);
                    self.__stack.drop_one_from_stack();
                    result
                }
                pub fn super_foo(&self, counter: Num) -> String {
                    let __class = self.__stack.pop_from_top_path();
                    match __class {
                        ClassName::A => {
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
