use proc_macro2::Span;
use quote::TokenStreamExt;
use syn::punctuated::Punctuated;
use syn::GenericArgument;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::str::FromStr;
use syn::PathArguments;
use syn::Ident;
use syn::Type;
use syn::Data;
use syn::Token;
use syn::DataStruct;
use syn::Fields;
use syn::Field;
use syn::DeriveInput;
use syn::parse_macro_input;
use quote::quote;

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);


    let name = input.ident;
    let buildername = Ident::new(&format!("{}Builder", name), Span::call_site());

    let data = input.data;
    let named_puc = match data{
         Data::Struct(s) => {
             match s.fields{
                 Fields::Named(fields) => {
                     fields.named
                 },
                _ => unreachable!{}
             }
         },
         _ => unreachable!{}
    };

    let mut optional_fields = vec![];
    let mut other_fields = vec![];
    for field in named_puc.iter(){
        match field.ty {
            Type::Path(ref tp) => {
                let segs = &tp.path.segments;
                if tp.qself.is_none() && segs.len() == 1 && &segs[0].ident.to_string() == "Option"{
                    optional_fields.push(field.clone());
                }else{
                    other_fields.push(field.clone());
                }
            }
            _ => {
                    other_fields.push(field.clone());
                    }
        }
    }

    // pack to opt
    for field in other_fields.iter_mut(){
        let mut ts = TokenStream::new();
        ts.extend(TokenStream::from_str("Option<").unwrap());
        field.ty.to_tokens(&mut ts);
        ts.extend(TokenStream::from_str(">").unwrap());
        let op_ty = syn::parse2(ts).unwrap();
        field.ty = op_ty;
    }
    let mut all_fields = other_fields;
    all_fields.append(&mut optional_fields);

    use std::iter::FromIterator;
    let fields = Punctuated::<Field, Token![,]>::from_iter(all_fields);
    //

    let mut funcs = vec![];
    for f in fields.iter(){
        let ident = f.ident.as_ref().unwrap().clone();
        let mut tp = f.ty.clone();
        let ty = if let Type::Path(ref mut p) = tp {
            match p.path.segments[0].arguments {
                PathArguments::AngleBracketed(ref a) => {
                    match a.args[0]{
                        GenericArgument::Type(ref t) => t.clone(),
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }else{
            unreachable!()
        };

        let fn_token = quote!{
            fn #ident(&mut self, #ident: #ty) -> &mut Self{
                self.#ident = Some(#ident);
                self
            }
        };
        funcs.push(fn_token);
    }
    let mut fun_tokens = TokenStream::new();
    fun_tokens.append_all(funcs);
    eprintln!("{}", fun_tokens.to_token_stream().to_string());


    let expanded = quote!{

        pub struct #buildername{
            #fields
        }

        impl #name {
            pub fn builder() -> #buildername{
                #buildername{
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }

            }
        }

        impl #buildername {
            #fun_tokens

            pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                use std::io::{Error, ErrorKind};
                let e = Error::new(ErrorKind::Other, "oh no!");
                if self.executable.is_none() || self.executable.is_none(){
                    return Result::<#name, Box<dyn std::error::Error>>::Err(Box::new(e));
                }
                let ret = #name {
                    executable: self.executable.take().unwrap(),
                    args: self.args.take().unwrap_or(vec![]),
                    env: self.env.take().unwrap_or(vec![]),
                    current_dir: self.current_dir.take().unwrap()
                };
                Ok(ret)
            }

        }

    };


    proc_macro::TokenStream::from(expanded)
}
