use c3_lang_parser::RustPackageDef;
use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro]
pub fn c3_lang(item: TokenStream) -> TokenStream {
    let rust_ast: RustPackageDef = syn::parse2(item.into()).unwrap();
    let c3_ast = c3_lang_parser::build_package_def(&rust_ast);
    c3_ast.to_token_stream().into()
}
