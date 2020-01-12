use proc_macro2::TokenStream;
use quote::quote;

use yarte_dom::DOMFmt;

use super::{CodeGen, EachCodeGen, IfElseCodeGen, HIR};

pub struct HTMLCodeGen;

impl EachCodeGen for HTMLCodeGen {}

impl IfElseCodeGen for HTMLCodeGen {}
impl CodeGen for HTMLCodeGen {
    fn gen(&self, v: Vec<HIR>) -> TokenStream {
        gen(self, v)
    }
}

pub struct HTMLMinCodeGen;
impl EachCodeGen for HTMLMinCodeGen {}
impl IfElseCodeGen for HTMLMinCodeGen {}

impl CodeGen for HTMLMinCodeGen {
    fn gen(&self, v: Vec<HIR>) -> TokenStream {
        let dom: DOMFmt = v.into();
        gen(self, dom.0)
    }
}

fn gen<C>(codegen: &C, v: Vec<HIR>) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => quote!(_fmt.write_str(#a)?;),
            Safe(a) => quote!(::std::fmt::Display::fmt(&(#a), _fmt)?;),
            Expr(a) => quote!(::yarte::Render::render(&(#a), _fmt)?;),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}
