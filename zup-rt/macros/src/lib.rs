#![deny(warnings)]

extern crate proc_macro;

use proc_macro::TokenStream;
use std::collections::HashSet;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse, parse_macro_input, spanned::Spanned, AttrStyle, Attribute, Item, ItemFn, ItemStatic,
    PathArguments, ReturnType, Stmt, Type, Visibility,
};

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    // check the function signature
    let valid_signature =
        check_signature(&f) && f.sig.inputs.is_empty() && is_bottom(&f.sig.output);

    if !valid_signature {
        return parse::Error::new(
            f.span(),
            "`#[entry]` function must have signature `fn() -> !`",
        )
        .to_compile_error()
        .into();
    }

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let attrs = f.attrs;
    let (statics, stmts) = match extract_static_muts(f.block.stmts) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let (args, items, vals) = locals(&statics);

    let ident = f.sig.ident;
    quote!(
        #[allow(non_snake_case)]
        #[inline(always)]
        fn #ident(#(#args,)*) -> ! {
            #(#stmts)*

            #[export_name = "main"]
            #[link_section = ".main"]
            #(#attrs)*
            unsafe extern "C" fn __entry__() -> ! {
                #(#items;)*

                #ident(#(#vals,)*)
            }
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn exception(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let fspan = f.span();
    let valid_signature =
        check_signature(&f) && f.sig.inputs.is_empty() && is_bottom(&f.sig.output);

    if !valid_signature {
        return parse::Error::new(fspan, "This exception must have signature `fn() -> !`")
            .to_compile_error()
            .into();
    }

    let ident = f.sig.ident;

    let ident_s = ident.to_string();

    let attrs = f.attrs;
    let block = f.block;
    let stmts = block.stmts;

    quote!(
        #[allow(non_snake_case)]
        fn #ident() -> ! {
            // check that this exception actually exists
            zup_rt::Exception::#ident;

            #(#stmts)*

            #[export_name = #ident_s]
            #(#attrs)*
            unsafe extern "C" fn __exception__() {
                #ident()
            }
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let fspan = f.span();
    let valid_signature = check_signature(&f) && f.sig.inputs.is_empty() && is_unit(&f.sig.output);

    if !valid_signature {
        return parse::Error::new(
            fspan,
            "`#[interrupts]` handlers must have  signature `fn() `",
        )
        .to_compile_error()
        .into();
    }

    let ident = f.sig.ident;

    let ident_s = ident.to_string();
    let (ref cfgs, ref attrs) = extract_cfgs(f.attrs);
    let block = f.block;
    let stmts = block.stmts;

    let (statics, stmts) = match extract_static_muts(stmts) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let (_, items, vals) = locals(&statics);

    quote!(
        #[allow(non_snake_case)]
        #(#cfgs)*
        fn #ident() {
            #(#stmts)*

            #[export_name = #ident_s]
            #(#cfgs)*
            #(#attrs)*
            fn __interrupt__() {
                #(#items;)*

                // check that this interrupt actually exists
                zup_rt::Interrupt::#ident;

                #ident(#(#vals,)*)
            }
        }
    )
    .into()
}

fn locals(
    statics: &[ItemStatic],
) -> (
    // args
    Vec<proc_macro2::TokenStream>,
    // items
    Vec<proc_macro2::TokenStream>,
    // vals
    Vec<proc_macro2::TokenStream>,
) {
    let mut args = vec![];
    let mut items = vec![];
    let mut vals = vec![];
    for static_ in statics {
        let attrs = &static_.attrs;
        let ident = &static_.ident;
        let expr = &static_.expr;
        let ty = &static_.ty;

        args.push(quote!(#ident: &'static mut #ty));
        items.push(quote!(
            #(#attrs)*
            static mut #ident: #ty = #expr
        ));
        vals.push(quote!(&mut #ident));
    }

    (args, items, vals)
}

// NOTE copy-paste from cortex-r-rt-macros v0.1.3
fn extract_static_muts(stmts: Vec<Stmt>) -> Result<(Vec<ItemStatic>, Vec<Stmt>), parse::Error> {
    let mut istmts = stmts.into_iter();

    let mut seen = HashSet::new();
    let mut statics = vec![];
    let mut stmts = vec![];
    for stmt in istmts.by_ref() {
        match stmt {
            Stmt::Item(Item::Static(var)) => {
                if let syn::StaticMutability::Mut(..) = var.mutability {
                    if seen.contains(&var.ident) {
                        return Err(parse::Error::new(
                            var.ident.span(),
                            format!("the name `{}` is defined multiple times", var.ident),
                        ));
                    }

                    seen.insert(var.ident.clone());
                    statics.push(var);
                } else {
                    stmts.push(Stmt::Item(Item::Static(var)));
                }
            }
            _ => {
                stmts.push(stmt);
                break;
            }
        }
    }

    stmts.extend(istmts);

    Ok((statics, stmts))
}

fn eq(attr: &Attribute, name: &str) -> bool {
    attr.style == AttrStyle::Outer && attr.path().segments.len() == 1 && {
        let segment = attr.path().segments.first().unwrap();
        segment.arguments == PathArguments::None && segment.ident == name
    }
}

fn extract_cfgs(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<Attribute>) {
    let mut cfgs = vec![];
    let mut not_cfgs = vec![];

    for attr in attrs {
        if eq(&attr, "cfg") {
            cfgs.push(attr);
        } else {
            not_cfgs.push(attr);
        }
    }

    (cfgs, not_cfgs)
}

/// checks that a function signature
///
/// - has no bounds (like where clauses)
/// - is not `async`
/// - is not `const`
/// - is not `unsafe`
/// - is not generic (has no type parametrs)
/// - is not variadic
/// - uses the Rust ABI (and not e.g. "C")
fn check_signature(item: &ItemFn) -> bool {
    item.vis == Visibility::Inherited
        && item.sig.constness.is_none()
        && item.sig.asyncness.is_none()
        && item.sig.abi.is_none()
        && item.sig.unsafety.is_none()
        && item.sig.generics.params.is_empty()
        && item.sig.generics.where_clause.is_none()
        && item.sig.variadic.is_none()
}

fn is_bottom(ret: &ReturnType) -> bool {
    match ret {
        ReturnType::Default => false,
        ReturnType::Type(_, ref ty) => matches!(**ty, Type::Never(..)),
    }
}

fn is_unit(ret: &ReturnType) -> bool {
    match ret {
        ReturnType::Default => true,
        ReturnType::Type(_, ref ty) => match **ty {
            Type::Tuple(ref ty) => ty.elems.is_empty(),
            _ => false,
        },
    }
}
