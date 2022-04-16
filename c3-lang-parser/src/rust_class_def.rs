use c3_lang_linearization::{Class, Fn, Var};
use quote::format_ident;
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Expr, Field, Fields, ImplItem, ImplItemConst,
    ImplItemMethod, ItemImpl, ItemStruct, Token, Visibility,
};

#[derive(Debug, PartialEq)]
pub struct RustClassDef {
    pub item_struct: ItemStruct,
    pub item_impl: Option<ItemImpl>,
}

impl Parse for RustClassDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_struct: ItemStruct = input.parse()?;
        let lookahead = input.lookahead1();
        let item_impl = if lookahead.peek(Token![impl]) {
            let item_impl: ItemImpl = input.parse()?;
            Some(item_impl)
        } else {
            None
        };
        Ok(RustClassDef {
            item_struct,
            item_impl,
        })
    }
}

impl RustClassDef {
    pub fn class(&self) -> Class {
        Class::from(self.item_struct.ident.to_string())
    }

    pub fn is_public(&self) -> bool {
        if let Visibility::Public(_) = self.item_struct.vis {
            true
        } else {
            false
        }
    }

    pub fn attrs(&self) -> Vec<Attribute> {
        self.item_struct.attrs.clone()
    }

    pub fn parents(&self) -> Vec<Class> {
        if self.item_impl.is_none() {
            return vec![];
        }

        let item_impl = self.item_impl.clone().unwrap();
        let items: Vec<ImplItem> = item_impl.items;
        let mut parents: Vec<Class> = vec![];
        for item in items.iter() {
            if let ImplItem::Const(item_const) = item {
                let item_const: &ImplItemConst = item_const;
                if item_const.ident == format_ident!("PARENTS") {
                    let expr = &item_const.expr;
                    if let Expr::Reference(expr_reference) = expr {
                        let expr = *expr_reference.expr.clone();
                        if let Expr::Array(expr_list) = expr {
                            let exprs: Punctuated<Expr, Token![,]> = expr_list.elems;
                            for expr in exprs.iter() {
                                if let Expr::Path(expr) = expr {
                                    let path = expr.path.clone();
                                    let segments = path.segments;
                                    if segments[0].ident == format_ident!("ClassName") {
                                        parents.push(Class::from(segments[1].ident.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        parents
    }

    pub fn functions(&self) -> Vec<Fn> {
        self.function_impls().into_iter().map(|x| x.0).collect()
    }

    pub fn function_impls(&self) -> Vec<(Fn, ImplItemMethod)> {
        let item_impl = self.item_impl.clone().unwrap();
        let items: Vec<ImplItem> = item_impl.items;
        let mut functions: Vec<(Fn, ImplItemMethod)> = vec![];
        for item in items.iter() {
            if let ImplItem::Method(method) = item {
                let name = method.sig.ident.to_string();
                functions.push((Fn::from(name), method.clone()));
            }
        }
        functions
    }

    pub fn variables(&self) -> Vec<Var> {
        self.variables_impl().into_iter().map(|x| x.0).collect()
    }

    pub fn variables_impl(&self) -> Vec<(Var, Field)> {
        let mut variables: Vec<(Var, Field)> = vec![];
        if let Fields::Named(fields) = &self.item_struct.fields {
            for field in &fields.named {
                let var = Var::from(field.ident.clone().unwrap().to_string());
                variables.push((var, field.clone()));
            }
        };
        variables
    }
}

#[cfg(test)]
mod tests {
    use c3_lang_linearization::Class;
    use quote::quote;
    use syn::parse_quote;

    use super::RustClassDef;

    #[test]
    fn test_rust_class_def_without_impl() {
        let input = quote! {
            struct A {}
        };
        let result: RustClassDef = syn::parse2(input).unwrap();
        let target = RustClassDef {
            item_struct: parse_quote!(
                struct A {}
            ),
            item_impl: None,
        };
        assert_eq!(result, target);
    }

    #[test]
    fn test_rust_class_def_with_impl() {
        let input = quote! {
            struct A {}
            impl A for B {}
        };
        let result: RustClassDef = syn::parse2(input).unwrap();
        let target = RustClassDef {
            item_struct: parse_quote!(
                struct A {}
            ),
            item_impl: Some(parse_quote!( impl A for B {} )),
        };
        assert_eq!(result, target);
    }

    #[test]
    fn test_rust_class_def_getters() {
        let input = quote! {
            #[derive(Default)]
            pub struct A {
                x: u32
            }

            impl A {
                const PARENTS: &'static [ClassName; 2usize] = &[
                    ClassName::X,
                    ClassName::Y
                ];

                pub fn k(&self) -> u32 { 4 }
            }
        };
        let result: RustClassDef = syn::parse2(input).unwrap();
        assert!(result.is_public());
        assert_eq!(result.class(), Class::from("A"));
        assert_eq!(result.parents(), vec![Class::from("X"), Class::from("Y")]);
        assert_eq!(result.attrs(), vec![parse_quote! { #[derive(Default)] }]);
    }
}
