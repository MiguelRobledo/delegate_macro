/*
** this monstruosity was brought to you by
** Miguel "Peppermint" Robledo, enjoy the mess.
**
** this is a very hacky way of doing delegation,
** please never use this in production in any way
** shape or form. this is very silly.
**
** this only works if the trait you are trying to
** delgate the methods of is defined in src/main.rs,
** this location is hardcoded, although there isn't
** any reason why you shouldn't be able to pass it
** as a parameter somewhere in the macro call other
** than laziness on my part.
**
** delegation should also work on normal impl blocks
** but this version doesn't because, again, laziness.
** this is just a proof of concept.
**
** in normal impl blocks the behavior would be similar
** except when you do `use x` without the `for y` in
** this case there are two options:
**     -create delegation methods for every single
**      method x defines.
**     -don't allow `use x` for anything other than
**      traits so you always know explitcitly what
**      methods are being created.
**
** it should also be possible to delegate functions
** that don't use self but this macro doesn't implement
** such a thing because, as has become a pattern by now,
** laziness. but if this is ever implemented in the
** compiler, it could simply create the method with a &self
** but not pass it to the method.
**
** i think this feature would be almost trivial to
** implement in the compilerand it allows for lots
** of code reuse without inheritance and much better
** ergonomics around composition.
*/

use std::fs::File;
use std::io::Read;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, FnArg, Ident, Item, ItemImpl, Pat, Result, Signature, Token, TraitItem,
};

extern crate proc_macro;

// this struct holds all the delegations in one block
struct Delegation {
    items: Punctuated<DelegationItem, Token![;]>,
}

impl Parse for Delegation {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Delegation {
            items: Punctuated::parse_terminated(input)?,
        })
    }
}

// this struct contains the data of each delegation
#[allow(dead_code)]
struct DelegationItem {
    use_token: Token![use],
    expr: Expr,
    for_token: Option<Token![for]>,
    items: Option<Punctuated<Ident, Token![,]>>,
}

impl Parse for DelegationItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(DelegationItem {
            use_token: input.parse()?,
            expr: input.parse()?,
            for_token: input.parse()?,
            items: match Punctuated::parse_separated_nonempty(input) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
        })
    }
}

#[proc_macro_attribute]
pub fn delegate(args: TokenStream, input: TokenStream) -> TokenStream {
    // we parse the impl Trait for x block
    let impl_item: ItemImpl = syn::parse(input).unwrap();

    // here we are just aliasing stuff mainly for quote!
    let items = impl_item.items;
    let trait_path = impl_item.trait_.expect("No trait declared").1;
    let trait_ident = &trait_path.segments.last().unwrap().ident;
    let type_name = impl_item.self_ty;

    // we parse the #[delegate(x)] block
    let del: Delegation = syn::parse(args).unwrap();

    let mut out = quote! {};
    for item in del.items {
        // more aliasing
        let expr = item.expr;

        // these are the functions we are trying to delegate
        let func_names = item.items.unwrap_or_default();

        // now comes the dumb part:
        //
        // we need to somehow get the signatures of the trait
        // methods for the trait we are trying to implement
        //
        // so naturally we parse the entire src/main.rs to try
        // to find a trait with the name we got before
        //
        // this limits us to only being able to delegate in
        // traits defined in src/main.rs and i cant think of
        // any way to get around this limitation
        let mut file = File::open("src/main.rs").expect("Unable to open file");
        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");
        let syntax = syn::parse_file(&src).expect("Unable to parse file");

        // this makes a vec of tuples that contain:
        //     -id of trait method
        //     -function signature
        //     -arguments to pass to the delegated function
        // for every function in specified in after the 'for'
        // token in our delegation declaration
        //
        // dont try to read it, just believe me
        let traits: Vec<(&Ident, &Signature, Punctuated<&Ident, Token![,]>)> = syntax
            .items
            .iter()
            .filter_map(|x| match x {
                Item::Trait(t) => {
                    if &t.ident == trait_ident {
                        Some(&t.items)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flatten()
            .filter_map(|x| match x {
                TraitItem::Method(m) => {
                    if func_names.is_empty() || func_names.iter().any(|x| *x == m.sig.ident) {
                        Some((
                            &m.sig.ident,
                            &m.sig,
                            m.sig
                                .inputs
                                .iter()
                                .filter_map(|x| match x {
                                    FnArg::Typed(t) => match &*t.pat {
                                        Pat::Ident(i) => Some(&i.ident),
                                        _ => None,
                                    },
                                    _ => None,
                                })
                                .collect(),
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        // here we extract each element into its own variable
        //
        // this is 100% not the correct way of doing this
        // i was just feeling lazy and didnt want to do it properly
        let func_names: Vec<&Ident> = traits.iter().map(|x| x.0).collect();
        let func_sigs: Vec<&Signature> = traits.iter().map(|x| x.1).collect();
        let func_args: Vec<&Punctuated<&Ident, Token![,]>> = traits.iter().map(|x| &x.2).collect();

        // and now we append methods to the output
        out = quote! {
            #out

            #(
                #func_sigs {
                    #expr . #func_names ( #func_args )
                }
            )*
        };
    }

    // we put all the functions in an impl block
    out = quote! {
        impl #trait_path for #type_name {
            #(#items)*

            #out
        }
    };

    // and there it is
    out.into()
}
