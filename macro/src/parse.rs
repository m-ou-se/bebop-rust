use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::cmp::max;
use std::convert::TryFrom;
use std::path::Path;

pub struct Parser<'a> {
    pub src: &'a str,
    pub file: &'a Path,
    pub crate_path: Ident,
}

impl<'a> Parser<'a> {
    pub fn skip_whitespace(&mut self) {
        loop {
            self.src = self.src.trim_start();
            if self.src.starts_with("//") {
                let n = self.src.find(&['\r', '\n'][..]).unwrap_or(self.src.len());
                self.src = &self.src[n..];
            } else if self.src.starts_with("/*") {
                let n = self.src.find("*/").map_or(self.src.len(), |n| n + 2);
                self.src = &self.src[n..];
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Option<&'a str> {
        if self.src.is_empty() {
            None
        } else {
            self.skip_whitespace();
            let n = self
                .src
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(self.src.len());
            let (token, rest) = self.src.split_at(max(n, 1));
            self.src = rest;
            Some(token)
        }
    }

    pub fn parse_identifier(&mut self) -> &'a str {
        match self.next_token() {
            Some(ident) if ident.starts_with(|c: char| c.is_alphabetic() || c == '_') => ident,
            Some(token) => panic!("expected identifier, but got {:?}", token),
            None => panic!("missing identifier"),
        }
    }

    pub fn parse_number(&mut self) -> u32 {
        match self.next_token() {
            Some(token) => {
                if let Some((_, hex)) = token.split_once("0x") {
                    u32::from_str_radix(hex, 16)
                        .unwrap_or_else(|_| panic!("invalid hexadecimal number"))
                } else {
                    u32::from_str_radix(token, 10).unwrap_or_else(|_| panic!("invalid number"))
                }
            }
            None => panic!("missing number"),
        }
    }

    pub fn parse_string_literal(&mut self) -> String {
        self.skip_whitespace();
        if let Some(s) = self.src.strip_prefix('\'') {
            match s.split_once('\'') {
                None => panic!("missing end of single quoted string literal"),
                Some((literal, rest)) => {
                    self.src = rest;
                    literal.into()
                }
            }
        } else if let Some(s) = self.src.strip_prefix('"') {
            match s.split_once('"') {
                None => panic!("missing end of double quoted string literal"),
                Some((literal, rest)) => {
                    self.src = rest;
                    literal.into()
                }
            }
        } else {
            panic!("expected string literal");
        }
    }

    pub fn parse_type(&mut self) -> TokenStream {
        let mut t = match self.next_token() {
            Some("map") => {
                self.expect("[");
                let key = self.parse_type();
                self.expect(",");
                let value = self.parse_type();
                self.expect("]");
                quote!(std::collections::HashMap<#key, #value>)
            }
            Some("array") => {
                self.expect("[");
                let element = self.parse_type();
                self.expect("]");
                quote!(Vec<#element>)
            }
            Some("string") => quote!(String),
            Some("bool") => quote!(bool),
            Some("byte") => quote!(u8),
            Some("uint8") => quote!(u8),
            Some("int8") => quote!(i8),
            Some("uint16") => quote!(u16),
            Some("int16") => quote!(i16),
            Some("uint32") => quote!(u32),
            Some("int32") => quote!(i32),
            Some("uint64") => quote!(u64),
            Some("int64") => quote!(i64),
            Some("float32") => quote!(f32),
            Some("float64") => quote!(f64),
            Some("date") => {
                let c = &self.crate_path;
                quote!(#c::Date)
            }
            Some("guid") => {
                let c = &self.crate_path;
                quote!(#c::Guid)
            }
            Some(name) => {
                let ident = Ident::new(name, Span::call_site());
                quote!(#ident)
            }
            None => panic!("missing type"),
        };
        while self.is_next("[") {
            self.expect("[");
            self.expect("]");
            t = quote!(Vec<#t>);
        }
        t
    }

    pub fn is_next(&mut self, next: &str) -> bool {
        self.skip_whitespace();
        self.src.starts_with(next)
    }

    pub fn expect(&mut self, expected: &str) {
        match self.next_token() {
            Some(token) if token == expected => {}
            Some(token) => panic!("expected {:?}, but got {:?}", expected, token),
            None => panic!("missing {:?}", expected),
        }
    }

    pub fn parse_opcode(&mut self) -> Option<u32> {
        if !self.is_next("[") {
            return None;
        }
        self.expect("[");
        self.expect("opcode");
        self.expect("(");
        self.skip_whitespace();
        let opcode = if self.src.starts_with(|c: char| c.is_numeric()) {
            self.parse_number()
        } else {
            let s = self.parse_string_literal();
            match <[u8; 4]>::try_from(s.as_bytes()) {
                Ok(bytes) => u32::from_le_bytes(bytes),
                Err(_) => panic!("opcodes must be four bytes"),
            }
        };
        self.expect(")");
        self.expect("]");
        Some(opcode)
    }

    pub fn parse_deprecated(&mut self) -> Option<TokenStream> {
        if !self.is_next("[") {
            return None;
        }
        self.expect("[");
        self.expect("deprecated");
        let attr = if self.is_next("]") {
            quote!(#[deprecated])
        } else {
            self.expect("(");
            self.skip_whitespace();
            let message = self.parse_string_literal();
            self.expect(")");
            quote!(#[deprecated = #message])
        };
        self.expect("]");
        Some(attr)
    }

    pub fn parse_definition(&mut self) -> (Ident, TokenStream) {
        let opcode = self.parse_opcode();
        let token = match self.next_token() {
            Some("readonly") => self.next_token(),
            t => t,
        };
        match token {
            Some("enum") => {
                if opcode.is_some() {
                    panic!("enums cannot have an opcode");
                }
                let name = Ident::new(self.parse_identifier(), Span::call_site());
                let mut names = Vec::new();
                let mut values = Vec::new();
                let mut attrs = Vec::new();
                self.expect("{");
                while !self.is_next("}") {
                    attrs.push(self.parse_deprecated());
                    names.push(Ident::new(self.parse_identifier(), Span::call_site()));
                    self.expect("=");
                    values.push(self.parse_number());
                    self.expect(";");
                }
                self.expect("}");
                let bebop = &self.crate_path;
                (
                    name.clone(),
                    quote!(
                        #[repr(u32)]
                        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
                        pub enum #name {
                            #(#attrs #names = #values,)*
                        }
                        impl #bebop::Bebop for #name {
                            fn read_from(reader: &mut #bebop::Reader) -> Result<Self, #bebop::Error> {
                                let value: u32 = reader.read()?;
                                match value {
                                    #(#values => Ok(Self::#names),)*
                                    _ => Err(#bebop::Error::UnknownEnumValue),
                                }
                            }
                            fn write_into(&self, writer: &mut #bebop::Writer) {
                                let value: u32 = match self {
                                    #(Self::#names => #values,)*
                                };
                                writer.write(&value);
                            }
                        }
                    ),
                )
            }
            Some("struct") => {
                let name = Ident::new(self.parse_identifier(), Span::call_site());
                self.expect("{");
                let mut types = Vec::new();
                let mut names = Vec::new();
                while !self.is_next("}") {
                    types.push(self.parse_type());
                    names.push(Ident::new(self.parse_identifier(), Span::call_site()));
                    self.expect(";");
                }
                self.expect("}");
                let bebop = &self.crate_path;
                let opcode = opcode.into_iter();
                (
                    name.clone(),
                    quote!(
                        #[derive(Clone, Debug, PartialEq)]
                        pub struct #name {
                            #(pub #names: #types,)*
                        }
                        #(
                            impl #bebop::Opcode for #name {
                                const OPCODE: u32 = #opcode;
                            }
                        )*
                        impl #bebop::Bebop for #name {
                            fn read_from(reader: &mut #bebop::Reader) -> Result<Self, #bebop::Error> {
                                Ok(Self {
                                    #(#names: reader.read()?,)*
                                })
                            }
                            fn write_into(&self, writer: &mut #bebop::Writer) {
                                #(writer.write(&self.#names);)*
                            }
                        }
                    ),
                )
            }
            Some("message") => {
                let name = Ident::new(self.parse_identifier(), Span::call_site());
                self.expect("{");
                let mut attrs = Vec::new();
                let mut indices = Vec::new();
                let mut types = Vec::new();
                let mut names = Vec::new();
                while !self.is_next("}") {
                    attrs.push(self.parse_deprecated());
                    let index = self.parse_number();
                    if index > 255 {
                        panic!("message field index must be <= 255, but got {}", index);
                    }
                    indices.push(index as u8);
                    self.expect("-");
                    self.expect(">");
                    types.push(self.parse_type());
                    names.push(Ident::new(self.parse_identifier(), Span::call_site()));
                    self.expect(";");
                }
                self.expect("}");
                let bebop = &self.crate_path;
                let opcode = opcode.into_iter();
                (
                    name.clone(),
                    quote!(
                        #[derive(Clone, Debug, Default, PartialEq)]
                        pub struct #name {
                            #(#attrs pub #names: Option<#types>,)*
                        }
                        #(
                            impl #bebop::Opcode for #name {
                                const OPCODE: u32 = #opcode;
                            }
                        )*
                        impl #bebop::Bebop for #name {
                            fn read_from(reader: &mut #bebop::Reader) -> Result<Self, #bebop::Error> {
                                let len: u32 = reader.read()?;
                                let bytes = reader.read_raw(len as usize)?;
                                let mut reader = #bebop::Reader::new(bytes);
                                let mut value = Self::default();
                                loop {
                                    match reader.read::<u8>()? {
                                        0 => break,
                                        #(#indices => value.#names = Some(reader.read()?),)*
                                        _ => break, // unknown field. skip to end of message
                                    }
                                }
                                Ok(value)
                            }
                            fn write_into(&self, writer: &mut #bebop::Writer) {
                                let offset = writer.bytes().len();
                                writer.write(&0u32); // placeholder for the size
                                #(
                                    if let Some(field) = &self.#names {
                                        writer.write::<u8>(&#indices);
                                        writer.write(field);
                                    }
                                )*
                                writer.write(&0u8);
                                // fill in the size in the placeholder we wrote before
                                let size = (writer.bytes().len() - 4 - offset) as u32;
                                writer.bytes_mut()[offset..][..4].copy_from_slice(&size.to_le_bytes());
                            }
                        }
                    ),
                )
            }
            Some("union") => {
                let name = Ident::new(self.parse_identifier(), Span::call_site());
                self.expect("{");
                let mut defs = TokenStream::new();
                let mut indices = Vec::new();
                let mut names = Vec::new();
                while !self.is_next("}") {
                    let index = self.parse_number();
                    if index > 255 {
                        panic!("union index must be <= 255, but got {}", index);
                    }
                    indices.push(index as u8);
                    self.expect("-");
                    self.expect(">");
                    let (field_name, field_def) = self.parse_definition();
                    names.push(field_name);
                    defs.extend(field_def);
                }
                self.expect("}");
                let bebop = &self.crate_path;
                let opcode = opcode.into_iter();
                (
                    name.clone(),
                    quote!(
                        #defs
                        #[derive(Clone, Debug, PartialEq)]
                        pub enum #name {
                            #(#names(#names),)*
                        }
                        #(
                            impl #bebop::Opcode for #name {
                                const OPCODE: u32 = #opcode;
                            }
                        )*
                        impl #bebop::Bebop for #name {
                            fn read_from(reader: &mut #bebop::Reader) -> Result<Self, #bebop::Error> {
                                let len: u32 = reader.read()?;
                                let tag: u8 = reader.read()?;
                                let bytes = reader.read_raw(len as usize)?;
                                let mut reader = #bebop::Reader::new(bytes);
                                match tag {
                                    #(#indices => Ok(Self::#names(reader.read()?)),)*
                                    _ => Err(#bebop::Error::UnknownUnionTag),
                                }
                            }
                            fn write_into(&self, writer: &mut #bebop::Writer) {
                                let offset = writer.bytes().len();
                                writer.write(&0u32); // placeholder for the size
                                match self {
                                    #(
                                        Self::#names(v) => {
                                            writer.write::<u8>(&#indices);
                                            writer.write(v);
                                        }
                                    )*
                                }
                                // fill in the size in the placeholder we wrote before
                                let size = (writer.bytes().len() - 5 - offset) as u32;
                                writer.bytes_mut()[offset..][..4].copy_from_slice(&size.to_le_bytes());
                            }
                        }
                    ),
                )
            }
            Some(token) => panic!("expected definition, but got {:?}", token),
            None => panic!("missing definiton"),
        }
    }

    pub fn parse_file(&mut self) -> TokenStream {
        let mut rust = TokenStream::new();
        loop {
            self.skip_whitespace();
            if self.src.is_empty() {
                break;
            }
            if self.src.starts_with("import") {
                self.expect("import");
                let file = self
                    .file
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(self.parse_string_literal());
                let src = match std::fs::read_to_string(&file) {
                    Ok(src) => src,
                    Err(e) => panic!("unable to open {:?}: {}", file, e),
                };
                let mut parser = Parser {
                    file: &file,
                    crate_path: self.crate_path.clone(),
                    src: &src,
                };
                rust.extend(parser.parse_file());
            } else {
                rust.extend(self.parse_definition().1);
            }
        }
        rust
    }
}
