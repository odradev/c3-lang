use quote::{format_ident, quote, ToTokens};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id {
    name: String,
}

impl From<&str> for Id {
    fn from(name: &str) -> Self {
        Id {
            name: String::from(name),
        }
    }
}

impl From<String> for Id {
    fn from(name: String) -> Self {
        Id { name }
    }
}

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.name
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl ToTokens for Id {
    fn to_tokens(&self, tokens: &mut quote::__private::TokenStream) {
        let name = format_ident!("{}", &self.name);
        tokens.extend(quote! {#name});
    }
}
