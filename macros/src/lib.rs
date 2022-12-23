use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::FoundCrate;
use quote::quote;
use syn::{parse_macro_input, Error, LitStr};

#[proc_macro]
pub fn minecraft(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = parse_macro_input!(input as LitStr);
    create_identifier(&quote!(MINECRAFT), &path).into()
}

#[proc_macro]
pub fn minestodon(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = parse_macro_input!(input as LitStr);
    create_identifier(&quote!(MINESTODON), &path).into()
}

fn create_identifier(namespace_const: &TokenStream, path: &LitStr) -> TokenStream {
    let valid = |c| matches!(c, 'a'..='z' | '0'..='9' | '.' | '-' | '_' | '/');
    if !path.value().chars().all(valid) {
        return Error::new(path.span(), "the path contains invalid characters").to_compile_error();
    }

    let main_crate = main_crate();
    quote! {
        unsafe {
            #main_crate::mc::Identifier::new_unchecked(
                #main_crate::mc::Identifier::#namespace_const,
                #path
            )
        }
    }
}

fn main_crate() -> TokenStream {
    let found = proc_macro_crate::crate_name("minestodon")
        .expect("failed to get the name of the main crate");
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
