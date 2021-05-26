mod parse;

use proc_macro::TokenStream as TokenStream1;
use quote::quote_spanned;
use std::path::Path;
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;

struct Input {
    crate_path: syn::Ident,
    file: syn::LitStr,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        Ok(Self {
            crate_path: input.parse()?,
            file: input.parse()?,
        })
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn read_bebop(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Input);

    let crate_path = input.crate_path;

    let file = if let Some(root) = std::env::var_os("CARGO_MANIFEST_DIR") {
        Path::new(&root).join(&input.file.value())
    } else {
        input.file.value().into()
    };

    let src = match std::fs::read_to_string(&file) {
        Ok(src) => src,
        Err(e) => {
            let msg = format!("unable to open {:?}: {}", file, e);
            return quote_spanned!(input.file.span() => compile_error! { #msg }).into();
        }
    };

    let mut parser = parse::Parser {
        file: &file,
        crate_path,
        src: &src,
    };

    parser.parse_file().into()
}
