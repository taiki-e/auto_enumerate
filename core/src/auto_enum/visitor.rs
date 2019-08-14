use syn::{
    token,
    visit_mut::{self, VisitMut},
    Arm, Attribute, Expr, ExprMacro, ExprMatch, ExprReturn, ExprTry, Item, Local, Stmt,
};

#[cfg(feature = "try_trait")]
use crate::utils::expr_call;
use crate::utils::{expr_compile_error, replace_expr};

use super::{parse_group, Attrs, AttrsMut, Context, Parent, VisitMode, EMPTY_ATTRS, NAME, NEVER};

// =============================================================================
// Visitor

#[derive(Clone, Copy, Default)]
struct Scope {
    /// in closures
    closure: bool,
    /// in try blocks
    try_block: bool,
    /// in the other `auto_enum` attributes
    foreign: bool,
}

pub(super) struct Visitor<'a> {
    cx: &'a mut Context,
    scope: Scope,
}

impl<'a> Visitor<'a> {
    pub(super) fn new(cx: &'a mut Context) -> Self {
        Self { cx, scope: Scope::default() }
    }

    fn find_remove_empty_attrs<A: AttrsMut>(&self, attrs: &mut A) {
        if !self.scope.foreign {
            EMPTY_ATTRS.iter().for_each(|ident| {
                attrs.find_remove_empty_attr(ident);
            });
        }
    }

    fn check_other_attr<A: Attrs>(&mut self, attrs: &A) {
        if attrs.any_attr(NAME) {
            self.scope.foreign = true;
            // Record whether other `auto_enum` attribute exists.
            self.cx.other_attr = true;
        }
    }

    /// `return` in functions or closures
    fn visit_return(&mut self, expr: &mut Expr) {
        debug_assert!(self.cx.visit_mode() == VisitMode::Return);

        if !self.scope.closure && !expr.any_empty_attr(NEVER) {
            // Desugar `return <expr>` into `return Enum::VariantN(<expr>)`.
            if let Expr::Return(ExprReturn { expr, .. }) = expr {
                self.cx.replace_boxed_expr(expr);
            }
        }
    }

    /// `?` operator in functions or closures
    fn visit_try(&mut self, expr: &mut Expr) {
        debug_assert!(self.cx.visit_mode() == VisitMode::Try);

        if !self.scope.try_block && !self.scope.closure && !expr.any_empty_attr(NEVER) {
            match &expr {
                // https://github.com/rust-lang/rust/blob/1.35.0/src/librustc/hir/lowering.rs#L4578-L4682

                // Desugar `ExprKind::Try`
                // from: `<expr>?`
                Expr::Try(ExprTry { expr: e, .. })
                    // Skip if `<expr>` is a marker macro.
                    if !self.cx.is_marker_expr(&**e) =>
                {
                    // into:
                    //
                    // match // If "try_trait" feature enabled
                    //       Try::into_result(<expr>)
                    //       // Otherwise
                    //       <expr>
                    // {
                    //     Ok(val) => val,
                    //     Err(err) => // If "try_trait" feature enabled
                    //                 return Try::from_error(Enum::VariantN(err)),
                    //                 // Otherwise
                    //                 return Err(Enum::VariantN(err)),
                    // }

                    replace_expr(expr, |expr| {
                        #[allow(unused_mut)]
                        let ExprTry { attrs, mut expr, .. } =
                            if let Expr::Try(expr) = expr { expr } else { unreachable!() };

                        #[cfg(feature = "try_trait")]
                        replace_expr(&mut *expr, |expr| {
                            expr_call(
                                Vec::new(),
                                syn::parse_quote!(::core::ops::Try::into_result),
                                expr,
                            )
                        });

                        let mut arms = Vec::with_capacity(2);
                        arms.push(syn::parse_quote!(::core::result::Result::Ok(val) => val,));

                        let err = self.cx.next_expr(syn::parse_quote!(err));
                        #[cfg(feature = "try_trait")]
                        arms.push(syn::parse_quote!(::core::result::Result::Err(err) => return ::core::ops::Try::from_error(#err),));
                        #[cfg(not(feature = "try_trait"))]
                        arms.push(syn::parse_quote!(::core::result::Result::Err(err) => return ::core::result::Result::Err(#err),));

                        Expr::Match(ExprMatch {
                            attrs,
                            match_token: token::Match::default(),
                            expr,
                            brace_token: token::Brace::default(),
                            arms,
                        })
                    })
                }
                _ => {}
            }
        }
    }

    /// Expression level marker (`marker!` macro)
    fn visit_marker_macro(&mut self, expr: &mut Expr) {
        debug_assert!(!self.scope.foreign || self.cx.marker.is_unique());

        match &expr {
            // Desugar `marker!(<expr>)` into `Enum::VariantN(<expr>)`.
            Expr::Macro(ExprMacro { mac, .. })
                // Skip if `marker!` is not a marker macro.
                if mac.path.is_ident(self.cx.marker.ident()) =>
            {
                replace_expr(expr, |expr| {
                    let expr = if let Expr::Macro(expr) = expr { expr } else { unreachable!() };
                    let args = syn::parse2(expr.mac.tokens).unwrap_or_else(|e| {
                        self.cx.error = true;
                        expr_compile_error(&e)
                    });

                    if self.cx.error {
                        args
                    } else {
                        self.cx.next_expr_with_attrs(expr.attrs, args)
                    }
                })
            }
            _ => {}
        }
    }
}

impl VisitMut for Visitor<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if !self.cx.error {
            let tmp = self.scope;
            self.check_other_attr(expr);

            match expr {
                Expr::Closure(_) => self.scope.closure = true,
                // `?` operator in try blocks are not supported.
                Expr::TryBlock(_) => self.scope.try_block = true,
                _ => {}
            }

            match self.cx.visit_mode() {
                VisitMode::Return => self.visit_return(expr),
                VisitMode::Try => self.visit_try(expr),
                VisitMode::Default => {}
            }

            visit_mut::visit_expr_mut(self, expr);

            if !self.scope.foreign || self.cx.marker.is_unique() {
                self.visit_marker_macro(expr);
                self.find_remove_empty_attrs(expr);
            }

            self.scope = tmp;
        }
    }

    fn visit_arm_mut(&mut self, arm: &mut Arm) {
        if !self.cx.error {
            visit_mut::visit_arm_mut(self, arm);
            self.find_remove_empty_attrs(arm);
        }
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        if !self.cx.error {
            let tmp = self.scope;
            self.check_other_attr(local);

            visit_mut::visit_local_mut(self, local);
            self.find_remove_empty_attrs(local);
            self.scope = tmp;
        }
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if !self.cx.error {
            visit_mut::visit_stmt_mut(self, stmt);
            visit_stmt_mut(stmt, self.cx);
        }
    }

    // Stop at item bounds
    fn visit_item_mut(&mut self, _item: &mut Item) {}
}

// =============================================================================
// FindTry

/// Find `?` operator.
pub(super) struct FindTry<'a> {
    cx: &'a Context,
    scope: Scope,
    pub(super) has: bool,
}

impl<'a> FindTry<'a> {
    pub(super) fn new(cx: &'a Context) -> Self {
        Self { cx, scope: Scope::default(), has: false }
    }
}

impl VisitMut for FindTry<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        let tmp = self.scope;

        if let Expr::Closure(_) = &expr {
            self.scope.closure = true;
        }

        if !self.scope.closure && !expr.any_empty_attr(NEVER) {
            if let Expr::Try(ExprTry { expr, .. }) = expr {
                // Skip if `<expr>` is a marker macro.
                if !self.cx.is_marker_expr(&**expr) {
                    self.has = true;
                }
            }
        }

        if expr.any_attr(NAME) {
            self.scope.foreign = true;
        }
        if !self.has {
            visit_mut::visit_expr_mut(self, expr);
        }

        self.scope = tmp;
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        let tmp = self.scope;

        if local.any_attr(NAME) {
            self.scope.foreign = true;
        }

        visit_mut::visit_local_mut(self, local);
        self.scope = tmp;
    }

    // Stop at item bounds
    fn visit_item_mut(&mut self, _item: &mut Item) {}
}

// =============================================================================
// Dummy visitor

pub(super) struct Dummy<'a> {
    cx: &'a mut Context,
}

impl<'a> Dummy<'a> {
    pub(super) fn new(cx: &'a mut Context) -> Self {
        Self { cx }
    }
}

impl VisitMut for Dummy<'_> {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if !self.cx.error {
            visit_mut::visit_stmt_mut(self, stmt);
            visit_stmt_mut(stmt, self.cx);
        }
    }

    // Stop at item bounds
    fn visit_item_mut(&mut self, _item: &mut Item) {}
}

fn visit_stmt_mut(stmt: &mut Stmt, cx: &mut Context) {
    // Stop at item bounds
    if let Stmt::Item(_) = stmt {
        return;
    }

    if let Some(Attribute { tokens, .. }) = stmt.find_remove_attr(NAME) {
        parse_group(tokens)
            .map(|x| Context::child(&stmt, x))
            .and_then(|mut cx| stmt.visit_parent(&mut cx))
            .unwrap_or_else(|e| {
                cx.error = true;
                *stmt = Stmt::Expr(expr_compile_error(&e));
            });
    }
}
