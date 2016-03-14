// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private))]

#[macro_use]
#[cfg(feature = "with-syntex")]
extern crate syntex_syntax as syntax;

#[macro_use]
#[cfg(not(feature = "with-syntex"))]
extern crate syntax;

use std::rc::Rc;

use syntax::errors::DiagnosticBuilder;
use syntax::ast::{self, TokenTree};
use syntax::codemap::{DUMMY_SP, Spanned, dummy_spanned};
use syntax::ext::base::ExtCtxt;
use syntax::parse::{self, classify, parse_tts_from_source_str, token};
use syntax::parse::parser::Parser;
use syntax::print::pprust;
use syntax::ptr::P;

pub trait ToTokens {
    fn to_tokens<'a, 'b, 'c>(&'a self,
                             _cx: &'b ExtCtxt<'c>)
                             -> Result<Vec<TokenTree>, DiagnosticBuilder<'c>>; }

impl ToTokens for TokenTree {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![self.clone()])
    }
}

impl<'a, T: ToTokens> ToTokens for &'a T {
    fn to_tokens<'b>(&self, cx: &ExtCtxt<'b>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'b>> {
        (**self).to_tokens(cx)
    }
}

impl<'a, T: ToTokens> ToTokens for &'a [T] {
    fn to_tokens<'b>(&self, cx: &ExtCtxt<'b>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'b>> {
        Ok(self.iter()
               .flat_map(|t| t.to_tokens(cx).unwrap().into_iter())
               .collect())
    }
}

impl<T: ToTokens> ToTokens for Vec<T> {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(self.iter().flat_map(|t| t.to_tokens(cx).unwrap()).collect())
    }
}

impl<T: ToTokens> ToTokens for Spanned<T> {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        // FIXME: use the span?
        self.node.to_tokens(cx)
    }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        match self {
            &Some(ref t) => t.to_tokens(cx),
            &None => Ok(Vec::new()),
        }
    }
}

impl ToTokens for ast::Ident {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(DUMMY_SP, token::Ident(*self, token::Plain))])
    }
}

impl ToTokens for ast::Path {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(DUMMY_SP,
                                      token::Interpolated(token::NtPath(Box::new(self.clone()))))])
    }
}

impl ToTokens for ast::Ty {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span,
                                      token::Interpolated(token::NtTy(P(self.clone()))))])
    }
}

impl ToTokens for P<ast::Ty> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span, token::Interpolated(token::NtTy(self.clone())))])
    }
}

impl ToTokens for P<ast::Block> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span,
                                      token::Interpolated(token::NtBlock(self.clone())))])
    }
}

impl ToTokens for P<ast::Item> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span, token::Interpolated(token::NtItem(self.clone())))])
    }
}

impl ToTokens for P<ast::ImplItem> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span,
                                      token::Interpolated(token::NtImplItem(self.clone())))])
    }
}

impl ToTokens for P<ast::TraitItem> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span,
                                      token::Interpolated(token::NtTraitItem(self.clone())))])
    }
}

impl ToTokens for ast::Generics {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        let s = pprust::generics_to_string(self);

        parse_tts_from_source_str("<quote expansion>".to_string(),
                                  s,
                                  cx.cfg(),
                                  cx.parse_sess())
    }
}

impl ToTokens for ast::WhereClause {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        let s = pprust::to_string(|s| s.print_where_clause(&self));

        parse_tts_from_source_str("<quote expansion>".to_string(),
                                  s,
                                  cx.cfg(),
                                  cx.parse_sess())
    }
}

impl ToTokens for ast::Stmt {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        let mut tts = vec![
            ast::TokenTree::Token(self.span, token::Interpolated(token::NtStmt(P(self.clone()))))
        ];

        // Some statements require a trailing semicolon.
        if classify::stmt_ends_with_semi(&self.node) {
            tts.push(ast::TokenTree::Token(self.span, token::Semi));
        }

        Ok(tts)
    }
}

impl ToTokens for P<ast::Expr> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span, token::Interpolated(token::NtExpr(self.clone())))])
    }
}

impl ToTokens for P<ast::Pat> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(self.span, token::Interpolated(token::NtPat(self.clone())))])
    }
}

impl ToTokens for ast::Arm {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(DUMMY_SP, token::Interpolated(token::NtArm(self.clone())))])
    }
}

macro_rules! impl_to_tokens_slice {
    ($t: ty, $sep: expr) => {
        impl ToTokens for [$t] {
            fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
                let mut v = vec![];
                for (i, x) in self.iter().enumerate() {
                    if i > 0 {
                        v.extend($sep.iter().cloned());
                    }
                    v.extend(try!(x.to_tokens(cx)));
                }
                
                Ok(v)
            }
        }
    };
}

impl_to_tokens_slice! { ast::Ty, [ast::TokenTree::Token(DUMMY_SP, token::Comma)] }
impl_to_tokens_slice! { P<ast::Item>, [] }

impl ToTokens for P<ast::MetaItem> {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Token(DUMMY_SP, token::Interpolated(token::NtMeta(self.clone())))])
    }
}

impl ToTokens for ast::Attribute {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        let mut r = vec![];
        // FIXME: The spans could be better
        r.push(ast::TokenTree::Token(self.span, token::Pound));
        if self.node.style == ast::AttrStyle::Inner {
            r.push(ast::TokenTree::Token(self.span, token::Not));
        }
        r.push(ast::TokenTree::Delimited(self.span,
                                         Rc::new(ast::Delimited {
                                             delim: token::Bracket,
                                             open_span: self.span,
                                             tts: try!(self.node.value.to_tokens(cx)),
                                             close_span: self.span,
                                         })));
        Ok(r)
    }
}

impl ToTokens for str {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        let lit = ast::LitKind::Str(token::intern_and_get_ident(self), ast::StrStyle::Cooked);
        dummy_spanned(lit).to_tokens(cx)
    }
}

impl ToTokens for () {
    fn to_tokens<'a>(&self, _cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        Ok(vec![ast::TokenTree::Delimited(DUMMY_SP,
                                          Rc::new(ast::Delimited {
                                              delim: token::Paren,
                                              open_span: DUMMY_SP,
                                              tts: vec![],
                                              close_span: DUMMY_SP,
                                          }))])
    }
}

impl ToTokens for ast::Lit {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        // FIXME: This is wrong
        P(ast::Expr {
            id: ast::DUMMY_NODE_ID,
            node: ast::ExprKind::Lit(P(self.clone())),
            span: DUMMY_SP,
            attrs: None,
        })
            .to_tokens(cx)
    }
}

impl ToTokens for bool {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        dummy_spanned(ast::LitKind::Bool(*self)).to_tokens(cx)
    }
}

impl ToTokens for char {
    fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
        dummy_spanned(ast::LitKind::Char(*self)).to_tokens(cx)
    }
}

macro_rules! impl_to_tokens_int {
    (signed, $t:ty, $tag:expr) => (
        impl ToTokens for $t {
            fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
                let val = if *self < 0 {
                    -self
                } else {
                    *self
                };
                let lit = ast::LitKind::Int(val as u64, ast::LitIntType::Signed($tag));
                dummy_spanned(lit).to_tokens(cx)
            }
        }
    );
    (unsigned, $t:ty, $tag:expr) => (
        impl ToTokens for $t {
            fn to_tokens<'a>(&self, cx: &ExtCtxt<'a>) -> Result<Vec<TokenTree>, DiagnosticBuilder<'a>> {
                let lit = ast::LitKind::Int(*self as u64, ast::LitIntType::Unsigned($tag));
                dummy_spanned(lit).to_tokens(cx)
            }
        }
    );
}

impl_to_tokens_int! { signed, isize, ast::IntTy::Is }
impl_to_tokens_int! { signed, i8,  ast::IntTy::I8 }
impl_to_tokens_int! { signed, i16, ast::IntTy::I16 }
impl_to_tokens_int! { signed, i32, ast::IntTy::I32 }
impl_to_tokens_int! { signed, i64, ast::IntTy::I64 }

impl_to_tokens_int! { unsigned, usize, ast::UintTy::Us }
impl_to_tokens_int! { unsigned, u8,   ast::UintTy::U8 }
impl_to_tokens_int! { unsigned, u16,  ast::UintTy::U16 }
impl_to_tokens_int! { unsigned, u32,  ast::UintTy::U32 }
impl_to_tokens_int! { unsigned, u64,  ast::UintTy::U64 }

pub trait ExtParseUtils {
    fn parse_item(&self, s: String) -> P<ast::Item>;
    fn parse_expr(&self, s: String) -> P<ast::Expr>;
    fn parse_stmt(&self, s: String) -> ast::Stmt;
    fn parse_tts(&self, s: String) -> Vec<ast::TokenTree>;
}

// TODO: add proper error handling instead of .expect().
impl<'a> ExtParseUtils for ExtCtxt<'a> {
    fn parse_item(&self, s: String) -> P<ast::Item> {
        parse::parse_item_from_source_str("<quote expansion>".to_string(),
                                          s,
                                          self.cfg(),
                                          self.parse_sess())
            .expect("parse error (syntax error)")
            .expect("parse error (no item found)")
    }

    fn parse_stmt(&self, s: String) -> ast::Stmt {
        parse::parse_stmt_from_source_str("<quote expansion>".to_string(),
                                          s,
                                          self.cfg(),
                                          self.parse_sess())
            .expect("parse error (syntax error)")
            .expect("parse error (no item found)")
    }

    fn parse_expr(&self, s: String) -> P<ast::Expr> {
        parse::parse_expr_from_source_str("<quote expansion>".to_string(),
                                          s,
                                          self.cfg(),
                                          self.parse_sess())
            .expect("parse error")
    }

    fn parse_tts(&self, s: String) -> Vec<ast::TokenTree> {
        parse::parse_tts_from_source_str("<quote expansion>".to_string(),
                                         s,
                                         self.cfg(),
                                         self.parse_sess())
            .expect("parse error")
    }
}

pub fn parse_expr_panic(parser: &mut Parser) -> P<ast::Expr> {
    panictry!(parser.parse_expr())
}

pub fn parse_item_panic(parser: &mut Parser) -> Option<P<ast::Item>> {
    panictry!(parser.parse_item())
}

pub fn parse_pat_panic(parser: &mut Parser) -> P<ast::Pat> {
    panictry!(parser.parse_pat())
}

pub fn parse_arm_panic(parser: &mut Parser) -> ast::Arm {
    panictry!(parser.parse_arm())
}

pub fn parse_ty_panic(parser: &mut Parser) -> P<ast::Ty> {
    panictry!(parser.parse_ty())
}

pub fn parse_stmt_panic(parser: &mut Parser) -> Option<ast::Stmt> {
    panictry!(parser.parse_stmt())
}

pub fn parse_attribute_panic(parser: &mut Parser, permit_inner: bool) -> ast::Attribute {
    panictry!(parser.parse_attribute(permit_inner))
}
