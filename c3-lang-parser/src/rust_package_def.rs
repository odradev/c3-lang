use super::RustClassDef;
use syn::{
    parse::{Parse, ParseStream, Peek},
    Item, Token,
};

#[derive(Debug, PartialEq)]
pub struct RustPackageDef {
    pub other_code: Vec<Item>,
    pub classes: Vec<RustClassDef>,
}

impl Parse for RustPackageDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut classes = vec![];
        let mut other_code: Vec<Item> = Vec::new();
        loop {
            if peek_with_pub(&input, Token![type])
                || peek_with_pub(&input, Token![mod])
                || peek_with_pub(&input, Token![use])
                || peek_with_pub(&input, Token![impl])
            {
                let item: Item = input.parse()?;
                other_code.push(item);
                continue;
            }

            if peek_with_pub(&input, Token![struct]) || input.peek(Token![#]) {
                let item: RustClassDef = input.parse()?;
                classes.push(item);
                continue;
            }

            if input.is_empty() {
                break;
            } else {
                return Err(input.error(format!("Unknown keyword. {}", input.to_string())));
            }
        }
        Ok(RustPackageDef {
            other_code,
            classes,
        })
    }
}

fn peek_with_pub<T: Peek>(input: &ParseStream, token: T) -> bool {
    input.peek(Token![pub]) && input.peek2(token) || input.peek(token)
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::RustPackageDef;

    #[test]
    fn test_rust_package_def_1() {
        let input = quote! {
            use c3;
            type A = X;
            pub type B = Y;
            mod a {}
            pub mod b {}
            struct K {}
            pub struct G {}
            impl G {}
        };
        let result: RustPackageDef = syn::parse2(input).unwrap();
        let target = RustPackageDef {
            other_code: vec![
                parse_quote! { use c3; },
                parse_quote! { type A = X; },
                parse_quote! { pub type B = Y; },
                parse_quote! { mod a { } },
                parse_quote! { pub mod b { } },
            ],
            classes: vec![
                parse_quote! { struct K {} },
                parse_quote! { pub struct G {} impl G {} },
            ],
        };
        assert_eq!(result, target);
    }
}
