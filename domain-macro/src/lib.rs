extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Field, Type};

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
    let data = &ast.data;
    let metadata_fields: Vec<&syn::Field> = match data {
        Data::Struct(s) => s
            .fields
            .iter()
            .filter(|f| {
                if f.ident.is_none() {
                    return false;
                }
                if let Type::Path(tp) = &f.ty {
                    let segment = &tp.path.segments;
                    if segment.len() == 1 {
                        return segment[0].ident == "Metadata";
                    }
                }
                false
            })
            .collect(),
        _ => {
            panic!("only struct allowed brah")
        }
    };
    if metadata_fields.len() != 1 {
        panic!("exactly one metadata field allowed for struct");
    }
    let metadata_field = metadata_fields.get(0).unwrap();
    let field_ident = metadata_field.ident.as_ref().unwrap();
    let gen = quote! {
        impl WithMetadata for #name {
           fn domain_metadata_mut(&mut self) -> &mut Metadata {
                &mut self.#field_ident
           }
        }
    };
    gen.into()
}
