use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use yarte_hir::{Struct, HIR};

use crate::{CodeGen, EachCodeGen, IfElseCodeGen};

pub struct FixedCodeGen<'a, T: CodeGen> {
    codegen: T,
    s: &'a Struct<'a>,
}

impl<'a, T: CodeGen> FixedCodeGen<'a, T> {
    pub fn new<'n>(codegen: T, s: &'n Struct) -> FixedCodeGen<'n, T> {
        FixedCodeGen { codegen, s }
    }

    #[inline]
    fn template(&mut self, nodes: Vec<HIR>, tokens: &mut TokenStream) {
        let nodes = self.codegen.gen(nodes);
        tokens.extend(self.s.implement_head(
            quote!(yarte::TemplateFixedTrait),
            &quote!(
                unsafe fn call(&self, buf: &mut [u8]) -> Option<usize> {
                    let buf_ptr = buf.as_mut_ptr();
                    let mut buf_cur = 0;
                    macro_rules! __yarte_write_bytes {
                        ($b:expr, $len:expr) => {
                            if buf.len() < buf_cur + $len{
                                return None;
                            } else {
                                for i in $b {
                                    buf_ptr.add(buf_cur).write(*i);
                                    buf_cur += 1;
                                }
                            }
                        };
                    }
                    #nodes
                    Some(buf_cur)
                }
            ),
        ));
    }
}

impl<'a, T: CodeGen> CodeGen for FixedCodeGen<'a, T> {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();

        self.template(v, &mut tokens);

        tokens
    }
}

pub struct TextFixedCodeGen(pub &'static str);

impl EachCodeGen for TextFixedCodeGen {}
impl IfElseCodeGen for TextFixedCodeGen {}

impl CodeGen for TextFixedCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let mut tokens = TokenStream::new();
        let parent = format_ident!("{}", self.0);
        for i in v {
            use HIR::*;
            tokens.extend(match i {
                Local(a) => quote!(#a),
                Lit(a) => {
                    let len = a.len();
                    quote!(__yarte_write_bytes!(#a.as_bytes(), #len);)
                },
                Safe(a) | Expr(a) => quote!(buf_cur += #parent::RenderSafe::render(&(#a), std::slice::from_raw_parts_mut(buf_ptr.add(buf_cur), buf.len() - buf_cur))?;),
                Each(a) => self.gen_each(*a),
                IfElse(a) => self.gen_if_else(*a),
            });
        }
        tokens
    }
}

fn gen<C>(codegen: &mut C, v: Vec<HIR>, parent: &str) -> TokenStream
where
    C: CodeGen + EachCodeGen + IfElseCodeGen,
{
    let mut tokens = TokenStream::new();
    let parent = format_ident!("{}", parent);
    for i in v {
        use HIR::*;
        tokens.extend(match i {
            Local(a) => quote!(#a),
            Lit(a) => {
                let len = a.len();
                quote!(__yarte_write_bytes!(#a.as_bytes(), #len);)
            },
            Safe(a) => quote!(buf_cur += #parent::RenderSafe::render(&(#a), std::slice::from_raw_parts_mut(buf_ptr.add(buf_cur), buf.len() - buf_cur))?;),
            Expr(a) => quote!(buf_cur += #parent::RenderFixed::render(&(#a), std::slice::from_raw_parts_mut(buf_ptr.add(buf_cur), buf.len() - buf_cur))?;),
            Each(a) => codegen.gen_each(*a),
            IfElse(a) => codegen.gen_if_else(*a),
        })
    }
    tokens
}

pub struct HTMLFixedCodeGen(pub &'static str);

impl EachCodeGen for HTMLFixedCodeGen {}

impl IfElseCodeGen for HTMLFixedCodeGen {}
impl CodeGen for HTMLFixedCodeGen {
    fn gen(&mut self, v: Vec<HIR>) -> TokenStream {
        let parent = self.0;
        gen(self, v, parent)
    }
}
