extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(WithMetadataMacro)]
pub fn with_metadata_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_with_metadata_macro(&ast)
}
fn impl_with_metadata_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl WithMetadata for #name {
           fn domain_metadata(&mut self) -> &mut Metadata {
                &mut self.domain_metadata
           }
        }
    };
    gen.into()
}
