use crate::{Register, RustClassDef, RustPackageDef};
use c3_lang_linearization::{c3_linearization, Class, Fn, C3};
use syn::{FnArg, ImplItemMethod, ReturnType};

use super::c3_ast::{ClassDef, ClassFnImpl, ClassNameDef, FnDef, PackageDef, VarDef};

// --- Package Builder ---

pub fn build_package_def(rust_package: &RustPackageDef) -> PackageDef {
    let other_code = rust_package.other_code.clone();
    let class_name = build_class_name_def(rust_package);
    let classes = build_classes(rust_package);

    PackageDef {
        other_code,
        class_name,
        classes,
    }
}

// --- Top Level Builders ---

fn build_class_name_def(rust_package: &RustPackageDef) -> ClassNameDef {
    ClassNameDef {
        classes: rust_package
            .classes
            .iter()
            .map(RustClassDef::class)
            .collect(),
    }
}

fn build_classes(rust_package: &RustPackageDef) -> Vec<ClassDef> {
    let c3 = build_c3(rust_package);
    let register = build_register(rust_package);
    let mut classes = vec![];
    for class in &rust_package.classes {
        if class.is_public() {
            classes.push(build_class(class, &c3, &register));
        }
    }
    classes
}

fn build_class(rust_class: &RustClassDef, c3: &C3, register: &Register) -> ClassDef {
    let class = rust_class.class();
    let variables = build_variables(&class, c3, register);
    let functions = build_functions(&class, c3, register);
    ClassDef {
        struct_attrs: rust_class.struct_attrs(),
        impl_attrs: rust_class.impl_attrs(),
        class: class.clone(),
        path: c3.path(&class).unwrap(),
        variables,
        functions,
    }
}

fn build_variables(class: &Class, c3: &C3, register: &Register) -> Vec<VarDef> {
    let mut variables = vec![];
    for variable in c3.variables(class) {
        let field = register.get_var(variable);
        variables.push(VarDef {
            ident: field.ident.unwrap().clone(),
            ty: field.ty,
        });
    }
    variables
}

fn build_functions(class: &Class, c3: &C3, register: &Register) -> Vec<FnDef> {
    let mut functions = vec![];
    for function in &c3.functions(class) {
        functions.push(build_function(function, register));
    }
    functions
}

fn build_function(fun: &Fn, register: &Register) -> FnDef {
    let first_impl: &ImplItemMethod = &register.get_first_impl(fun);
    let args = get_args_from_method(first_impl);
    let ret = get_ret_from_method(first_impl);
    let implementations = get_class_function_impls(fun, register);
    FnDef {
        attrs: first_impl.attrs.clone(),
        name: fun.clone(),
        args,
        ret,
        implementations,
    }
}

// --- Utils ---

fn get_args_from_method(method: &ImplItemMethod) -> Vec<FnArg> {
    method.sig.inputs.clone().into_iter().collect()
}

fn get_ret_from_method(method: &ImplItemMethod) -> ReturnType {
    method.sig.output.clone()
}

fn get_class_function_impls(fun: &Fn, register: &Register) -> Vec<ClassFnImpl> {
    let mut result = vec![];
    for (class, impl_method) in register.get(fun) {
        result.push(ClassFnImpl {
            class,
            fun: fun.clone(),
            implementation: impl_method.block,
        });
    }
    result
}

fn build_c3(rust_package: &RustPackageDef) -> C3 {
    let mut input = C3::new();
    for class in &rust_package.classes {
        input.add(class.class(), class.parents());
    }
    let mut output = c3_linearization(input).unwrap();
    for class in &rust_package.classes {
        output.register_fns(class.class(), class.functions());
        output.register_vars(class.class(), class.variables());
    }
    output
}

fn build_register(rust_package: &RustPackageDef) -> Register {
    let mut register = Register::default();
    for class in &rust_package.classes {
        for (name, implementation) in class.function_impls() {
            register.add(class.class(), name, implementation);
        }
        for (var, field) in class.variables_impl() {
            register.add_var(var, field);
        }
    }
    register
}

#[cfg(test)]
pub mod tests {
    use c3_lang_linearization::{Class, Fn};
    use syn::parse_quote;

    use crate::c3_ast::*;
    use crate::test_utils::test_structs;
    use crate::RustPackageDef;

    use super::build_package_def;

    pub fn test_c3_code() -> RustPackageDef {
        parse_quote! {
            pub type Num = u32;

            struct A {
                x: u32
            }

            impl A {
                pub fn bar(&self, counter: Num) -> String {
                    let label = format!("A::bar({})", counter);
                    if counter == 0 {
                        label
                    } else {
                        format!("{} {}", label, self.foo(counter - 1))
                    }
                }

                #[test]
                pub fn foo(&self, counter: Num) -> String {
                    let label = format!("A::foo({})", counter);
                    if counter == 0 {
                        label
                    } else {
                        format!("{} {}", label, self.bar(counter - 1))
                    }
                }
            }

            #[derive(Debug)]
            pub struct B {
            }

            #[cfg(target_os = "linux")] 
            impl B {
                pub const PARENTS: &'static [ClassName; 1] = &[ClassName::A];

                pub fn bar(&self, counter: Num) -> String {
                    let label = format!("B::bar({})", counter);
                    if counter == 0 {
                        label
                    } else {
                        format!("{} {}", label, self.super_bar(counter - 1))
                    }
                }
            }
        }
    }

    pub fn test_c3_ast() -> PackageDef {
        PackageDef {
            other_code: vec![parse_quote! { pub type Num = u32; }],
            class_name: ClassNameDef {
                classes: vec![Class::from("A"), Class::from("B")],
            },
            classes: vec![ClassDef {
                struct_attrs: vec![parse_quote! { #[derive(Debug)] }],
                impl_attrs: vec![parse_quote! { #[cfg(target_os = "linux")] }],
                class: Class::from("B"),
                path: vec![Class::from("B"), Class::from("A")],
                variables: vec![VarDef {
                    ident: parse_quote! { x },
                    ty: parse_quote! { u32 },
                }],
                functions: vec![
                    FnDef {
                        attrs: Vec::new(),
                        name: Fn::from("bar"),
                        args: vec![parse_quote! { &self }, parse_quote! { counter: Num }],
                        ret: parse_quote! { -> String },
                        implementations: vec![
                            ClassFnImpl {
                                class: Class::from("A"),
                                fun: Fn::from("bar"),
                                implementation: parse_quote! {{
                                    let label = format!("A::bar({})", counter);
                                    if counter == 0 {
                                        label
                                    } else {
                                        format!("{} {}", label, self.foo(counter - 1))
                                    }
                                }},
                            },
                            ClassFnImpl {
                                class: Class::from("B"),
                                fun: Fn::from("bar"),
                                implementation: parse_quote! {{
                                    let label = format!("B::bar({})", counter);
                                    if counter == 0 {
                                        label
                                    } else {
                                        format!("{} {}", label, self.super_bar(counter - 1))
                                    }
                                }},
                            },
                        ],
                    },
                    FnDef {
                        attrs: vec![parse_quote! { #[test] }],
                        name: Fn::from("foo"),
                        args: vec![parse_quote! { &self }, parse_quote! { counter: Num }],
                        ret: parse_quote! { -> String },
                        implementations: vec![ClassFnImpl {
                            class: Class::from("A"),
                            fun: Fn::from("foo"),
                            implementation: parse_quote! {{
                                let label = format!("A::foo({})", counter);
                                if counter == 0 {
                                    label
                                } else {
                                    format!("{} {}", label, self.bar(counter - 1))
                                }
                            }},
                        }],
                    },
                ],
            }],
        }
    }

    #[test]
    fn test_building_package() {
        let result = build_package_def(&test_c3_code());
        let target = test_c3_ast();
        test_structs(result, target);
    }
}
