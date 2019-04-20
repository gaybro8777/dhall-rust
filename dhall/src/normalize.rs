#![allow(non_snake_case)]
use std::collections::BTreeMap;
use std::rc::Rc;

use dhall_core::context::Context;
use dhall_core::*;
use dhall_generator as dhall;

use crate::expr::*;

type InputSubExpr = SubExpr<X, Normalized<'static>>;
type OutputSubExpr = SubExpr<X, X>;

impl<'a> Typed<'a> {
    pub fn normalize(self) -> Normalized<'a> {
        Normalized(normalize(self.0), self.1, self.2)
    }
    /// Pretends this expression is normalized. Use with care.
    #[allow(dead_code)]
    pub fn skip_normalize(self) -> Normalized<'a> {
        Normalized(
            self.0.unroll().squash_embed(|e| e.0.clone()),
            self.1,
            self.2,
        )
    }
}

fn apply_builtin(
    ctx: NormalizationContext,
    b: Builtin,
    args: &[WHNF],
) -> Option<WHNF> {
    use dhall_core::Builtin::*;
    use WHNF::*;

    let whnf_to_expr = |v: &WHNF| v.clone().normalize_to_expr().embed_absurd();
    let now_to_expr =
        |v: &Now| v.clone().normalize().normalize_to_expr().embed_absurd();

    let ret = match (b, args) {
        (OptionalNone, [t]) => EmptyOptionalLit(Now::from_whnf(t.clone())),
        (NaturalIsZero, [NaturalLit(n)]) => BoolLit(*n == 0),
        (NaturalEven, [NaturalLit(n)]) => BoolLit(*n % 2 == 0),
        (NaturalOdd, [NaturalLit(n)]) => BoolLit(*n % 2 != 0),
        (NaturalToInteger, [NaturalLit(n)]) => {
            Expr(rc(ExprF::IntegerLit(*n as isize)))
        }
        (NaturalShow, [NaturalLit(n)]) => {
            TextLit(vec![InterpolatedTextContents::Text(n.to_string())])
        }
        (ListLength, [_, EmptyListLit(_)]) => NaturalLit(0),
        (ListLength, [_, NEListLit(xs)]) => NaturalLit(xs.len()),
        (ListHead, [_, EmptyListLit(n)]) => EmptyOptionalLit(n.clone()),
        (ListHead, [_, NEListLit(xs)]) => {
            NEOptionalLit(xs.first().unwrap().clone())
        }
        (ListLast, [_, EmptyListLit(n)]) => EmptyOptionalLit(n.clone()),
        (ListLast, [_, NEListLit(xs)]) => {
            NEOptionalLit(xs.last().unwrap().clone())
        }
        (ListReverse, [_, EmptyListLit(n)]) => EmptyListLit(n.clone()),
        (ListReverse, [_, NEListLit(xs)]) => {
            let xs = xs.iter().cloned().rev().collect();
            NEListLit(xs)
        }
        (ListIndexed, [_, EmptyListLit(t)]) => {
            // TODO: avoid passing through Exprs
            let t = now_to_expr(t);
            EmptyListLit(Now::new(
                ctx,
                dhall::subexpr!({ index : Natural, value : t }),
            ))
        }
        (ListIndexed, [_, NEListLit(xs)]) => {
            let xs = xs
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, e)| {
                    let i = NaturalLit(i);
                    let mut kvs = BTreeMap::new();
                    kvs.insert("index".into(), Now::from_whnf(i));
                    kvs.insert("value".into(), e);
                    Now::from_whnf(RecordLit(kvs))
                })
                .collect();
            NEListLit(xs)
        }
        // fold/build fusion
        (ListBuild, [_, WHNF::AppliedBuiltin(_, ListFold, args)])
            if args.len() == 2 =>
        {
            args.get(1).unwrap().clone()
        }
        (ListFold, [_, WHNF::AppliedBuiltin(_, ListBuild, args)])
            if args.len() == 2 =>
        {
            args.get(1).unwrap().clone()
        }
        // TODO: avoid passing through Exprs
        (ListBuild, [a0, g]) => g
            .clone()
            .app(AppliedBuiltin(ctx.clone(), List, vec![]).app(a0.clone()))
            .app(normalize_whnf(ctx, {
                let a0 = whnf_to_expr(a0);
                let a1 = shift0(1, &"x".into(), &a0);
                dhall::subexpr!(λ(x : a0) -> λ(xs : List a1) -> [ x ] # xs)
            }))
            .app(EmptyListLit(Now::from_whnf(a0.clone()))),
        (ListFold, [_, EmptyListLit(_), _, _, nil]) => nil.clone(),
        (ListFold, [_, NEListLit(xs), _, cons, nil]) => {
            let mut v = nil.clone();
            for x in xs.iter().rev() {
                v = cons.clone().app(x.clone().normalize()).app(v);
            }
            v
        }
        // fold/build fusion
        (OptionalBuild, [_, WHNF::AppliedBuiltin(_, OptionalFold, args)])
            if args.len() == 2 =>
        {
            args.get(1).unwrap().clone()
        }
        (OptionalFold, [_, WHNF::AppliedBuiltin(_, OptionalBuild, args)])
            if args.len() == 2 =>
        {
            args.get(1).unwrap().clone()
        }
        // TODO: avoid passing through Exprs
        (OptionalBuild, [a0, g]) => g
            .clone()
            .app(AppliedBuiltin(ctx.clone(), Optional, vec![]).app(a0.clone()))
            .app(normalize_whnf(ctx, {
                let a0 = whnf_to_expr(a0);
                dhall::subexpr!(λ(x: a0) -> Some x)
            }))
            .app(EmptyOptionalLit(Now::from_whnf(a0.clone()))),
        (OptionalFold, [_, EmptyOptionalLit(_), _, _, nothing]) => {
            nothing.clone()
        }
        (OptionalFold, [_, NEOptionalLit(x), _, just, _]) => {
            just.clone().app(x.clone().normalize())
        }
        // fold/build fusion
        (NaturalBuild, [WHNF::AppliedBuiltin(_, NaturalFold, args)])
            if args.len() == 1 =>
        {
            args.get(0).unwrap().clone()
        }
        (NaturalFold, [WHNF::AppliedBuiltin(_, NaturalBuild, args)])
            if args.len() == 1 =>
        {
            args.get(0).unwrap().clone()
        }
        // TODO: avoid passing through Exprs
        (NaturalBuild, [g]) => g
            .clone()
            .app(AppliedBuiltin(ctx.clone(), Natural, vec![]))
            .app(normalize_whnf(
                ctx,
                dhall::subexpr!((λ(x : Natural) -> x + 1)),
            ))
            .app(NaturalLit(0)),
        (NaturalFold, [NaturalLit(0), _, _, zero]) => zero.clone(),
        (NaturalFold, [NaturalLit(n), t, succ, zero]) => {
            let fold = AppliedBuiltin(ctx, NaturalFold, vec![])
                .app(NaturalLit(n - 1))
                .app(t.clone())
                .app(succ.clone())
                .app(zero.clone());
            succ.clone().app(fold)
        }
        _ => return None,
    };
    Some(ret)
}

#[derive(Debug, Clone)]
enum EnvItem {
    Expr(WHNF),
    Skip(usize),
}

#[derive(Debug, Clone)]
struct NormalizationContext(Rc<Context<Label, EnvItem>>);

impl NormalizationContext {
    fn new() -> Self {
        NormalizationContext(Rc::new(Context::new()))
    }
    fn insert(&self, x: &Label, e: WHNF) -> Self {
        NormalizationContext(Rc::new(
            self.0.insert(x.clone(), EnvItem::Expr(e)),
        ))
    }
    fn skip(&self, x: &Label) -> Self {
        NormalizationContext(Rc::new(
            self.0
                .map(|l, e| {
                    use EnvItem::*;
                    match e {
                        Expr(e) => Expr(e.clone().shift0(1, x)),
                        Skip(n) if l == x => Skip(*n + 1),
                        Skip(n) => Skip(*n),
                    }
                })
                .insert(x.clone(), EnvItem::Skip(0)),
        ))
    }
    fn lookup(&self, var: &V<Label>) -> WHNF {
        let V(x, n) = var;
        match self.0.lookup(x, *n) {
            Some(EnvItem::Expr(e)) => e.clone(),
            Some(EnvItem::Skip(m)) => {
                WHNF::Expr(rc(ExprF::Var(V(x.clone(), *m))))
            }
            None => WHNF::Expr(rc(ExprF::Var(V(x.clone(), *n)))),
        }
    }
}

/// A semantic value. This is partially redundant with `dhall_core::Expr`, on purpose. `Expr` should
/// be limited to syntactic expressions: either written by the user or meant to be printed.
/// The rule is the following: we must _not_ construct values of type `Expr` while normalizing or
/// typechecking, but only construct `WHNF`s.
///
/// WHNFs usually store subexpressions unnormalized, to enable lazy normalization. They approximate
/// what's called Weak Head Normal-Form (WHNF). This means that the expression is normalized as
/// little as possible, but just enough to know the first constructor of the normal form. This is
/// identical to full normalization for simple types like integers, but for e.g. a record literal
/// this means knowing just the field names.
///
/// Each variant captures the relevant contexts when it is constructed. Conceptually each
/// subexpression has its own context, but in practice some contexts can be shared when sensible, to
/// avoid unnecessary allocations.
#[derive(Debug, Clone)]
enum WHNF {
    Lam(Label, Now, (NormalizationContext, InputSubExpr)),
    AppliedBuiltin(NormalizationContext, Builtin, Vec<WHNF>),
    BoolLit(bool),
    NaturalLit(Natural),
    EmptyOptionalLit(Now),
    NEOptionalLit(Now),
    EmptyListLit(Now),
    NEListLit(Vec<Now>),
    RecordLit(BTreeMap<Label, Now>),
    UnionType(NormalizationContext, BTreeMap<Label, Option<InputSubExpr>>),
    UnionConstructor(
        NormalizationContext,
        Label,
        BTreeMap<Label, Option<InputSubExpr>>,
    ),
    UnionLit(
        Label,
        Now,
        (NormalizationContext, BTreeMap<Label, Option<InputSubExpr>>),
    ),
    TextLit(Vec<InterpolatedTextContents<Now>>),
    Expr(OutputSubExpr),
}

impl WHNF {
    /// Convert the value back to a (normalized) syntactic expression
    fn normalize_to_expr(self) -> OutputSubExpr {
        match self {
            WHNF::Lam(x, t, (ctx, e)) => {
                let ctx2 = ctx.skip(&x);
                rc(ExprF::Lam(
                    x.clone(),
                    t.normalize().normalize_to_expr(),
                    normalize_whnf(ctx2, e).normalize_to_expr(),
                ))
            }
            WHNF::AppliedBuiltin(_ctx, b, args) => {
                let mut e = WHNF::Expr(rc(ExprF::Builtin(b)));
                for v in args {
                    e = e.app(v);
                }
                e.normalize_to_expr()
            }
            WHNF::BoolLit(b) => rc(ExprF::BoolLit(b)),
            WHNF::NaturalLit(n) => rc(ExprF::NaturalLit(n)),
            WHNF::EmptyOptionalLit(n) => {
                rc(ExprF::EmptyOptionalLit(n.normalize().normalize_to_expr()))
            }
            WHNF::NEOptionalLit(n) => {
                rc(ExprF::NEOptionalLit(n.normalize().normalize_to_expr()))
            }
            WHNF::EmptyListLit(n) => {
                rc(ExprF::EmptyListLit(n.normalize().normalize_to_expr()))
            }
            WHNF::NEListLit(elts) => rc(ExprF::NEListLit(
                elts.into_iter()
                    .map(|n| n.normalize().normalize_to_expr())
                    .collect(),
            )),
            WHNF::RecordLit(kvs) => rc(ExprF::RecordLit(
                kvs.into_iter()
                    .map(|(k, v)| (k, v.normalize().normalize_to_expr()))
                    .collect(),
            )),
            WHNF::UnionType(ctx, kts) => rc(ExprF::UnionType(
                kts.into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            v.map(|v| {
                                normalize_whnf(ctx.clone(), v)
                                    .normalize_to_expr()
                            }),
                        )
                    })
                    .collect(),
            )),
            WHNF::UnionConstructor(ctx, l, kts) => {
                let kts = kts
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            v.map(|v| {
                                normalize_whnf(ctx.clone(), v)
                                    .normalize_to_expr()
                            }),
                        )
                    })
                    .collect();
                rc(ExprF::Field(rc(ExprF::UnionType(kts)), l))
            }
            WHNF::UnionLit(l, v, (kts_ctx, kts)) => rc(ExprF::UnionLit(
                l,
                v.normalize().normalize_to_expr(),
                kts.into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            v.map(|v| {
                                normalize_whnf(kts_ctx.clone(), v)
                                    .normalize_to_expr()
                            }),
                        )
                    })
                    .collect(),
            )),
            WHNF::TextLit(elts) => {
                fn normalize_textlit(
                    elts: Vec<InterpolatedTextContents<Now>>,
                ) -> InterpolatedText<OutputSubExpr> {
                    elts.into_iter()
                        .flat_map(|contents| {
                            use InterpolatedTextContents::{Expr, Text};
                            let new_interpolated = match contents {
                                Expr(n) => match n.normalize() {
                                    WHNF::TextLit(elts2) => {
                                        normalize_textlit(elts2)
                                    }
                                    e => InterpolatedText::from((
                                        String::new(),
                                        vec![(
                                            e.normalize_to_expr(),
                                            String::new(),
                                        )],
                                    )),
                                },
                                Text(s) => InterpolatedText::from(s),
                            };
                            new_interpolated.into_iter()
                        })
                        .collect()
                }

                rc(ExprF::TextLit(normalize_textlit(elts)))
            }
            WHNF::Expr(e) => e,
        }
    }

    /// Apply to a value
    fn app(self, val: WHNF) -> WHNF {
        match self {
            WHNF::Lam(x, _, (ctx, e)) => {
                let ctx2 = ctx.insert(&x, val);
                normalize_whnf(ctx2, e)
            }
            WHNF::AppliedBuiltin(ctx, b, mut args) => {
                args.push(val);
                match apply_builtin(ctx.clone(), b, &args) {
                    Some(v) => v,
                    None => WHNF::AppliedBuiltin(ctx, b, args),
                }
            }
            WHNF::UnionConstructor(ctx, l, kts) => {
                WHNF::UnionLit(l, Now::from_whnf(val), (ctx, kts))
            }
            // Can't do anything useful, convert to expr
            v => WHNF::Expr(rc(ExprF::App(
                v.normalize_to_expr(),
                val.normalize_to_expr(),
            ))),
        }
    }

    fn shift0(self, delta: isize, label: &Label) -> Self {
        match self {
            WHNF::Lam(x, t, (ctx, e)) => WHNF::Lam(
                x,
                t.shift0(delta, label),
                (ctx, shift(delta, &V(label.clone(), 1), &e)),
            ),
            WHNF::AppliedBuiltin(ctx, b, args) => WHNF::AppliedBuiltin(
                ctx,
                b,
                args.into_iter().map(|e| e.shift0(delta, label)).collect(),
            ),
            WHNF::Expr(e) => WHNF::Expr(shift0(delta, label, &e)),
            WHNF::BoolLit(b) => WHNF::BoolLit(b),
            WHNF::NaturalLit(n) => WHNF::NaturalLit(n),
            WHNF::EmptyOptionalLit(n) => {
                WHNF::EmptyOptionalLit(n.shift0(delta, label))
            }
            WHNF::NEOptionalLit(n) => {
                WHNF::NEOptionalLit(n.shift0(delta, label))
            }
            WHNF::EmptyListLit(n) => WHNF::EmptyListLit(n.shift0(delta, label)),
            WHNF::NEListLit(elts) => WHNF::NEListLit(
                elts.into_iter().map(|n| n.shift0(delta, label)).collect(),
            ),
            WHNF::RecordLit(kvs) => WHNF::RecordLit(
                kvs.into_iter()
                    .map(|(k, v)| (k, v.shift0(delta, label)))
                    .collect(),
            ),
            WHNF::UnionType(ctx, kts) => WHNF::UnionType(
                ctx,
                kts.into_iter()
                    .map(|(k, v)| (k, v.map(|v| shift0(delta, label, &v))))
                    .collect(),
            ),
            WHNF::UnionConstructor(ctx, l, kts) => {
                let kts = kts
                    .into_iter()
                    .map(|(k, v)| (k, v.map(|v| shift0(delta, label, &v))))
                    .collect();
                WHNF::UnionConstructor(ctx, l, kts)
            }
            WHNF::UnionLit(l, v, (kts_ctx, kts)) => WHNF::UnionLit(
                l,
                v.shift0(delta, label),
                (
                    kts_ctx,
                    kts.into_iter()
                        .map(|(k, v)| (k, v.map(|v| shift0(delta, label, &v))))
                        .collect(),
                ),
            ),
            WHNF::TextLit(elts) => WHNF::TextLit(
                elts.into_iter()
                    .map(|contents| {
                        use InterpolatedTextContents::{Expr, Text};
                        match contents {
                            Expr(n) => Expr(n.shift0(delta, label)),
                            Text(s) => Text(s),
                        }
                    })
                    .collect(),
            ),
        }
    }
}

/// Normalize-on-write smart container. Contains either a (partially) normalized value or a
/// non-normalized value alongside a normalization context.
/// The name is a pun on std::borrow::Cow.
#[derive(Debug, Clone)]
enum Now {
    Normalized(Box<WHNF>),
    Unnormalized(NormalizationContext, InputSubExpr),
}

impl Now {
    fn new(ctx: NormalizationContext, e: InputSubExpr) -> Now {
        Now::Unnormalized(ctx, e)
    }

    fn from_whnf(v: WHNF) -> Now {
        Now::Normalized(Box::new(v))
    }

    fn normalize(self) -> WHNF {
        match self {
            Now::Normalized(v) => *v,
            Now::Unnormalized(ctx, e) => normalize_whnf(ctx, e),
        }
    }

    fn shift0(self, delta: isize, label: &Label) -> Self {
        match self {
            Now::Normalized(v) => {
                Now::Normalized(Box::new(v.shift0(delta, label)))
            }
            Now::Unnormalized(ctx, e) => {
                Now::Unnormalized(ctx, shift0(delta, label, &e))
            }
        }
    }
}

/// Reduces the imput expression to WHNF. See doc on `WHNF` for details.
fn normalize_whnf(ctx: NormalizationContext, expr: InputSubExpr) -> WHNF {
    let expr = match expr.as_ref() {
        ExprF::Var(v) => return ctx.lookup(&v),
        ExprF::Annot(x, _) => return normalize_whnf(ctx, x.clone()),
        ExprF::Note(_, e) => return normalize_whnf(ctx, e.clone()),
        // TODO: wasteful to retraverse everything
        ExprF::Embed(e) => return normalize_whnf(ctx, e.0.embed_absurd()),
        ExprF::Let(x, _, r, b) => {
            let r = normalize_whnf(ctx.clone(), r.clone());
            return normalize_whnf(ctx.insert(x, r), b.clone());
        }
        ExprF::Lam(x, t, e) => {
            return WHNF::Lam(
                x.clone(),
                Now::new(ctx.clone(), t.clone()),
                (ctx, e.clone()),
            )
        }
        ExprF::Builtin(b) => return WHNF::AppliedBuiltin(ctx, *b, vec![]),
        ExprF::BoolLit(b) => return WHNF::BoolLit(*b),
        ExprF::NaturalLit(n) => return WHNF::NaturalLit(*n),
        ExprF::OldOptionalLit(None, e) => {
            return WHNF::EmptyOptionalLit(Now::new(ctx, e.clone()))
        }
        ExprF::OldOptionalLit(Some(e), _) => {
            return WHNF::NEOptionalLit(Now::new(ctx, e.clone()))
        }
        ExprF::EmptyOptionalLit(e) => {
            return WHNF::EmptyOptionalLit(Now::new(ctx, e.clone()))
        }
        ExprF::NEOptionalLit(e) => {
            return WHNF::NEOptionalLit(Now::new(ctx, e.clone()))
        }
        ExprF::EmptyListLit(e) => {
            return WHNF::EmptyListLit(Now::new(ctx, e.clone()))
        }
        ExprF::NEListLit(elts) => {
            return WHNF::NEListLit(
                elts.iter()
                    .map(|e| Now::new(ctx.clone(), e.clone()))
                    .collect(),
            )
        }
        ExprF::RecordLit(kvs) => {
            return WHNF::RecordLit(
                kvs.iter()
                    .map(|(k, e)| (k.clone(), Now::new(ctx.clone(), e.clone())))
                    .collect(),
            )
        }
        ExprF::UnionType(kvs) => return WHNF::UnionType(ctx, kvs.clone()),
        ExprF::UnionLit(l, x, kts) => {
            return WHNF::UnionLit(
                l.clone(),
                Now::new(ctx.clone(), x.clone()),
                (ctx, kts.clone()),
            )
        }
        ExprF::TextLit(elts) => {
            return WHNF::TextLit(
                elts.iter()
                    .map(|contents| {
                        use InterpolatedTextContents::{Expr, Text};
                        match contents {
                            Expr(n) => Expr(Now::new(ctx.clone(), n.clone())),
                            Text(s) => Text(s.clone()),
                        }
                    })
                    .collect(),
            )
        }
        ExprF::BoolIf(b, e1, e2) => {
            let b = normalize_whnf(ctx.clone(), b.clone());
            match b {
                WHNF::BoolLit(true) => return normalize_whnf(ctx, e1.clone()),
                WHNF::BoolLit(false) => return normalize_whnf(ctx, e2.clone()),
                _ => expr,
            }
        }
        _ => expr,
    };

    // Recursively normalize all subexpressions
    let expr: ExprF<WHNF, Label, X, X> =
        expr.as_ref().map_ref_with_special_handling_of_binders(
            |e| normalize_whnf(ctx.clone(), e.clone()),
            |x, e| normalize_whnf(ctx.skip(x), e.clone()),
            X::clone,
            |_| unreachable!(),
            Label::clone,
        );

    normalize_last_layer(ctx, expr)
}

/// When all sub-expressions have been (partially) normalized, eval the remaining toplevel layer.
fn normalize_last_layer(
    ctx: NormalizationContext,
    expr: ExprF<WHNF, Label, X, X>,
) -> WHNF {
    use dhall_core::BinOp::*;
    use dhall_core::ExprF::*;

    let expr = match expr {
        App(v, a) => return v.app(a),

        BinOp(BoolAnd, WHNF::BoolLit(true), y) => return y,
        BinOp(BoolAnd, x, WHNF::BoolLit(true)) => return x,
        BinOp(BoolAnd, WHNF::BoolLit(false), _) => return WHNF::BoolLit(false),
        BinOp(BoolAnd, _, WHNF::BoolLit(false)) => return WHNF::BoolLit(false),
        BinOp(BoolOr, WHNF::BoolLit(true), _) => return WHNF::BoolLit(true),
        BinOp(BoolOr, _, WHNF::BoolLit(true)) => return WHNF::BoolLit(true),
        BinOp(BoolOr, WHNF::BoolLit(false), y) => return y,
        BinOp(BoolOr, x, WHNF::BoolLit(false)) => return x,
        BinOp(BoolEQ, WHNF::BoolLit(true), y) => return y,
        BinOp(BoolEQ, x, WHNF::BoolLit(true)) => return x,
        BinOp(BoolNE, WHNF::BoolLit(false), y) => return y,
        BinOp(BoolNE, x, WHNF::BoolLit(false)) => return x,
        BinOp(BoolEQ, WHNF::BoolLit(x), WHNF::BoolLit(y)) => {
            return WHNF::BoolLit(x == y)
        }
        BinOp(BoolNE, WHNF::BoolLit(x), WHNF::BoolLit(y)) => {
            return WHNF::BoolLit(x != y)
        }

        BinOp(NaturalPlus, WHNF::NaturalLit(0), y) => return y,
        BinOp(NaturalPlus, x, WHNF::NaturalLit(0)) => return x,
        BinOp(NaturalTimes, WHNF::NaturalLit(0), _) => {
            return WHNF::NaturalLit(0)
        }
        BinOp(NaturalTimes, _, WHNF::NaturalLit(0)) => {
            return WHNF::NaturalLit(0)
        }
        BinOp(NaturalTimes, WHNF::NaturalLit(1), y) => return y,
        BinOp(NaturalTimes, x, WHNF::NaturalLit(1)) => return x,
        BinOp(NaturalPlus, WHNF::NaturalLit(x), WHNF::NaturalLit(y)) => {
            return WHNF::NaturalLit(x + y)
        }
        BinOp(NaturalTimes, WHNF::NaturalLit(x), WHNF::NaturalLit(y)) => {
            return WHNF::NaturalLit(x * y)
        }

        BinOp(ListAppend, WHNF::EmptyListLit(_), y) => return y,
        BinOp(ListAppend, x, WHNF::EmptyListLit(_)) => return x,
        BinOp(ListAppend, WHNF::NEListLit(mut xs), WHNF::NEListLit(mut ys)) => {
            xs.append(&mut ys);
            return WHNF::NEListLit(xs);
        }
        BinOp(TextAppend, WHNF::TextLit(mut x), WHNF::TextLit(mut y)) => {
            x.append(&mut y);
            return WHNF::TextLit(x);
        }

        Field(WHNF::UnionType(ctx, kts), l) => {
            return WHNF::UnionConstructor(ctx, l, kts)
        }
        Field(WHNF::RecordLit(mut kvs), l) => {
            match kvs.remove(&l) {
                Some(r) => return r.normalize(),
                // Return ownership
                None => Field(WHNF::RecordLit(kvs), l),
            }
        }
        // Always simplify `x.{}` to `{}`
        Projection(_, ls) if ls.is_empty() => {
            return WHNF::RecordLit(std::collections::BTreeMap::new())
        }
        Projection(WHNF::RecordLit(mut kvs), ls) => {
            return WHNF::RecordLit(
                ls.into_iter()
                    .filter_map(|l| kvs.remove(&l).map(|x| (l, x)))
                    .collect(),
            )
        }
        Merge(
            WHNF::RecordLit(mut handlers),
            WHNF::UnionConstructor(ctor_ctx, l, kts),
            t,
        ) => match handlers.remove(&l) {
            Some(h) => return h.normalize(),
            // Return ownership
            None => Merge(
                WHNF::RecordLit(handlers),
                WHNF::UnionConstructor(ctor_ctx, l, kts),
                t,
            ),
        },
        Merge(
            WHNF::RecordLit(mut handlers),
            WHNF::UnionLit(l, v, (kts_ctx, kts)),
            t,
        ) => match handlers.remove(&l) {
            Some(h) => {
                let h = h.normalize();
                let v = v.normalize();
                return normalize_last_layer(ctx, App(h, v));
            }
            // Return ownership
            None => Merge(
                WHNF::RecordLit(handlers),
                WHNF::UnionLit(l, v, (kts_ctx, kts)),
                t,
            ),
        },
        expr => expr,
    };

    WHNF::Expr(rc(expr.map_ref_simple(|e| e.clone().normalize_to_expr())))
}

/// Reduce an expression to its normal form, performing beta reduction
///
/// `normalize` does not type-check the expression.  You may want to type-check
/// expressions before normalizing them since normalization can convert an
/// ill-typed expression into a well-typed expression.
///
/// However, `normalize` will not fail if the expression is ill-typed and will
/// leave ill-typed sub-expressions unevaluated.
///
fn normalize(e: InputSubExpr) -> OutputSubExpr {
    normalize_whnf(NormalizationContext::new(), e).normalize_to_expr()
}

#[cfg(test)]
mod spec_tests {
    #![rustfmt::skip]

    macro_rules! norm {
        ($name:ident, $path:expr) => {
            make_spec_test!(Normalization, Success, $name, $path);
        };
    }

    norm!(success_haskell_tutorial_access_0, "haskell-tutorial/access/0");
    norm!(success_haskell_tutorial_access_1, "haskell-tutorial/access/1");
    // norm!(success_haskell_tutorial_combineTypes_0, "haskell-tutorial/combineTypes/0");
    // norm!(success_haskell_tutorial_combineTypes_1, "haskell-tutorial/combineTypes/1");
    // norm!(success_haskell_tutorial_prefer_0, "haskell-tutorial/prefer/0");
    norm!(success_haskell_tutorial_projection_0, "haskell-tutorial/projection/0");


    norm!(success_prelude_Bool_and_0, "prelude/Bool/and/0");
    norm!(success_prelude_Bool_and_1, "prelude/Bool/and/1");
    norm!(success_prelude_Bool_build_0, "prelude/Bool/build/0");
    norm!(success_prelude_Bool_build_1, "prelude/Bool/build/1");
    norm!(success_prelude_Bool_even_0, "prelude/Bool/even/0");
    norm!(success_prelude_Bool_even_1, "prelude/Bool/even/1");
    norm!(success_prelude_Bool_even_2, "prelude/Bool/even/2");
    norm!(success_prelude_Bool_even_3, "prelude/Bool/even/3");
    norm!(success_prelude_Bool_fold_0, "prelude/Bool/fold/0");
    norm!(success_prelude_Bool_fold_1, "prelude/Bool/fold/1");
    norm!(success_prelude_Bool_not_0, "prelude/Bool/not/0");
    norm!(success_prelude_Bool_not_1, "prelude/Bool/not/1");
    norm!(success_prelude_Bool_odd_0, "prelude/Bool/odd/0");
    norm!(success_prelude_Bool_odd_1, "prelude/Bool/odd/1");
    norm!(success_prelude_Bool_odd_2, "prelude/Bool/odd/2");
    norm!(success_prelude_Bool_odd_3, "prelude/Bool/odd/3");
    norm!(success_prelude_Bool_or_0, "prelude/Bool/or/0");
    norm!(success_prelude_Bool_or_1, "prelude/Bool/or/1");
    norm!(success_prelude_Bool_show_0, "prelude/Bool/show/0");
    norm!(success_prelude_Bool_show_1, "prelude/Bool/show/1");
    // norm!(success_prelude_Double_show_0, "prelude/Double/show/0");
    // norm!(success_prelude_Double_show_1, "prelude/Double/show/1");
    // norm!(success_prelude_Integer_show_0, "prelude/Integer/show/0");
    // norm!(success_prelude_Integer_show_1, "prelude/Integer/show/1");
    // norm!(success_prelude_Integer_toDouble_0, "prelude/Integer/toDouble/0");
    // norm!(success_prelude_Integer_toDouble_1, "prelude/Integer/toDouble/1");
    norm!(success_prelude_List_all_0, "prelude/List/all/0");
    norm!(success_prelude_List_all_1, "prelude/List/all/1");
    norm!(success_prelude_List_any_0, "prelude/List/any/0");
    norm!(success_prelude_List_any_1, "prelude/List/any/1");
    norm!(success_prelude_List_build_0, "prelude/List/build/0");
    norm!(success_prelude_List_build_1, "prelude/List/build/1");
    norm!(success_prelude_List_concat_0, "prelude/List/concat/0");
    norm!(success_prelude_List_concat_1, "prelude/List/concat/1");
    norm!(success_prelude_List_concatMap_0, "prelude/List/concatMap/0");
    norm!(success_prelude_List_concatMap_1, "prelude/List/concatMap/1");
    norm!(success_prelude_List_filter_0, "prelude/List/filter/0");
    norm!(success_prelude_List_filter_1, "prelude/List/filter/1");
    norm!(success_prelude_List_fold_0, "prelude/List/fold/0");
    norm!(success_prelude_List_fold_1, "prelude/List/fold/1");
    norm!(success_prelude_List_fold_2, "prelude/List/fold/2");
    norm!(success_prelude_List_generate_0, "prelude/List/generate/0");
    norm!(success_prelude_List_generate_1, "prelude/List/generate/1");
    norm!(success_prelude_List_head_0, "prelude/List/head/0");
    norm!(success_prelude_List_head_1, "prelude/List/head/1");
    norm!(success_prelude_List_indexed_0, "prelude/List/indexed/0");
    norm!(success_prelude_List_indexed_1, "prelude/List/indexed/1");
    norm!(success_prelude_List_iterate_0, "prelude/List/iterate/0");
    norm!(success_prelude_List_iterate_1, "prelude/List/iterate/1");
    norm!(success_prelude_List_last_0, "prelude/List/last/0");
    norm!(success_prelude_List_last_1, "prelude/List/last/1");
    norm!(success_prelude_List_length_0, "prelude/List/length/0");
    norm!(success_prelude_List_length_1, "prelude/List/length/1");
    norm!(success_prelude_List_map_0, "prelude/List/map/0");
    norm!(success_prelude_List_map_1, "prelude/List/map/1");
    norm!(success_prelude_List_null_0, "prelude/List/null/0");
    norm!(success_prelude_List_null_1, "prelude/List/null/1");
    norm!(success_prelude_List_replicate_0, "prelude/List/replicate/0");
    norm!(success_prelude_List_replicate_1, "prelude/List/replicate/1");
    norm!(success_prelude_List_reverse_0, "prelude/List/reverse/0");
    norm!(success_prelude_List_reverse_1, "prelude/List/reverse/1");
    norm!(success_prelude_List_shifted_0, "prelude/List/shifted/0");
    norm!(success_prelude_List_shifted_1, "prelude/List/shifted/1");
    norm!(success_prelude_List_unzip_0, "prelude/List/unzip/0");
    norm!(success_prelude_List_unzip_1, "prelude/List/unzip/1");
    norm!(success_prelude_Natural_build_0, "prelude/Natural/build/0");
    norm!(success_prelude_Natural_build_1, "prelude/Natural/build/1");
    norm!(success_prelude_Natural_enumerate_0, "prelude/Natural/enumerate/0");
    norm!(success_prelude_Natural_enumerate_1, "prelude/Natural/enumerate/1");
    norm!(success_prelude_Natural_even_0, "prelude/Natural/even/0");
    norm!(success_prelude_Natural_even_1, "prelude/Natural/even/1");
    norm!(success_prelude_Natural_fold_0, "prelude/Natural/fold/0");
    norm!(success_prelude_Natural_fold_1, "prelude/Natural/fold/1");
    norm!(success_prelude_Natural_fold_2, "prelude/Natural/fold/2");
    norm!(success_prelude_Natural_isZero_0, "prelude/Natural/isZero/0");
    norm!(success_prelude_Natural_isZero_1, "prelude/Natural/isZero/1");
    norm!(success_prelude_Natural_odd_0, "prelude/Natural/odd/0");
    norm!(success_prelude_Natural_odd_1, "prelude/Natural/odd/1");
    norm!(success_prelude_Natural_product_0, "prelude/Natural/product/0");
    norm!(success_prelude_Natural_product_1, "prelude/Natural/product/1");
    // norm!(success_prelude_Natural_show_0, "prelude/Natural/show/0");
    // norm!(success_prelude_Natural_show_1, "prelude/Natural/show/1");
    norm!(success_prelude_Natural_sum_0, "prelude/Natural/sum/0");
    norm!(success_prelude_Natural_sum_1, "prelude/Natural/sum/1");
    // norm!(success_prelude_Natural_toDouble_0, "prelude/Natural/toDouble/0");
    // norm!(success_prelude_Natural_toDouble_1, "prelude/Natural/toDouble/1");
    // norm!(success_prelude_Natural_toInteger_0, "prelude/Natural/toInteger/0");
    // norm!(success_prelude_Natural_toInteger_1, "prelude/Natural/toInteger/1");
    norm!(success_prelude_Optional_all_0, "prelude/Optional/all/0");
    norm!(success_prelude_Optional_all_1, "prelude/Optional/all/1");
    norm!(success_prelude_Optional_any_0, "prelude/Optional/any/0");
    norm!(success_prelude_Optional_any_1, "prelude/Optional/any/1");
    // norm!(success_prelude_Optional_build_0, "prelude/Optional/build/0");
    // norm!(success_prelude_Optional_build_1, "prelude/Optional/build/1");
    norm!(success_prelude_Optional_concat_0, "prelude/Optional/concat/0");
    norm!(success_prelude_Optional_concat_1, "prelude/Optional/concat/1");
    norm!(success_prelude_Optional_concat_2, "prelude/Optional/concat/2");
    // norm!(success_prelude_Optional_filter_0, "prelude/Optional/filter/0");
    // norm!(success_prelude_Optional_filter_1, "prelude/Optional/filter/1");
    norm!(success_prelude_Optional_fold_0, "prelude/Optional/fold/0");
    norm!(success_prelude_Optional_fold_1, "prelude/Optional/fold/1");
    norm!(success_prelude_Optional_head_0, "prelude/Optional/head/0");
    norm!(success_prelude_Optional_head_1, "prelude/Optional/head/1");
    norm!(success_prelude_Optional_head_2, "prelude/Optional/head/2");
    norm!(success_prelude_Optional_last_0, "prelude/Optional/last/0");
    norm!(success_prelude_Optional_last_1, "prelude/Optional/last/1");
    norm!(success_prelude_Optional_last_2, "prelude/Optional/last/2");
    norm!(success_prelude_Optional_length_0, "prelude/Optional/length/0");
    norm!(success_prelude_Optional_length_1, "prelude/Optional/length/1");
    norm!(success_prelude_Optional_map_0, "prelude/Optional/map/0");
    norm!(success_prelude_Optional_map_1, "prelude/Optional/map/1");
    norm!(success_prelude_Optional_null_0, "prelude/Optional/null/0");
    norm!(success_prelude_Optional_null_1, "prelude/Optional/null/1");
    norm!(success_prelude_Optional_toList_0, "prelude/Optional/toList/0");
    norm!(success_prelude_Optional_toList_1, "prelude/Optional/toList/1");
    norm!(success_prelude_Optional_unzip_0, "prelude/Optional/unzip/0");
    norm!(success_prelude_Optional_unzip_1, "prelude/Optional/unzip/1");
    norm!(success_prelude_Text_concat_0, "prelude/Text/concat/0");
    norm!(success_prelude_Text_concat_1, "prelude/Text/concat/1");
    norm!(success_prelude_Text_concatMap_0, "prelude/Text/concatMap/0");
    norm!(success_prelude_Text_concatMap_1, "prelude/Text/concatMap/1");
    // norm!(success_prelude_Text_concatMapSep_0, "prelude/Text/concatMapSep/0");
    // norm!(success_prelude_Text_concatMapSep_1, "prelude/Text/concatMapSep/1");
    // norm!(success_prelude_Text_concatSep_0, "prelude/Text/concatSep/0");
    // norm!(success_prelude_Text_concatSep_1, "prelude/Text/concatSep/1");
    // norm!(success_prelude_Text_show_0, "prelude/Text/show/0");
    // norm!(success_prelude_Text_show_1, "prelude/Text/show/1");



    // norm!(success_remoteSystems, "remoteSystems");
    // norm!(success_simple_doubleShow, "simple/doubleShow");
    // norm!(success_simple_integerShow, "simple/integerShow");
    // norm!(success_simple_integerToDouble, "simple/integerToDouble");
    // norm!(success_simple_letlet, "simple/letlet");
    norm!(success_simple_listBuild, "simple/listBuild");
    norm!(success_simple_multiLine, "simple/multiLine");
    norm!(success_simple_naturalBuild, "simple/naturalBuild");
    norm!(success_simple_naturalPlus, "simple/naturalPlus");
    norm!(success_simple_naturalShow, "simple/naturalShow");
    norm!(success_simple_naturalToInteger, "simple/naturalToInteger");
    norm!(success_simple_optionalBuild, "simple/optionalBuild");
    norm!(success_simple_optionalBuildFold, "simple/optionalBuildFold");
    norm!(success_simple_optionalFold, "simple/optionalFold");
    // norm!(success_simple_sortOperator, "simple/sortOperator");
    // norm!(success_simplifications_and, "simplifications/and");
    // norm!(success_simplifications_eq, "simplifications/eq");
    // norm!(success_simplifications_ifThenElse, "simplifications/ifThenElse");
    // norm!(success_simplifications_ne, "simplifications/ne");
    // norm!(success_simplifications_or, "simplifications/or");


    norm!(success_unit_Bool, "unit/Bool");
    norm!(success_unit_Double, "unit/Double");
    norm!(success_unit_DoubleLiteral, "unit/DoubleLiteral");
    norm!(success_unit_DoubleShow, "unit/DoubleShow");
    // norm!(success_unit_DoubleShowValue, "unit/DoubleShowValue");
    norm!(success_unit_FunctionApplicationCapture, "unit/FunctionApplicationCapture");
    norm!(success_unit_FunctionApplicationNoSubstitute, "unit/FunctionApplicationNoSubstitute");
    norm!(success_unit_FunctionApplicationNormalizeArguments, "unit/FunctionApplicationNormalizeArguments");
    norm!(success_unit_FunctionApplicationSubstitute, "unit/FunctionApplicationSubstitute");
    norm!(success_unit_FunctionNormalizeArguments, "unit/FunctionNormalizeArguments");
    norm!(success_unit_FunctionTypeNormalizeArguments, "unit/FunctionTypeNormalizeArguments");
    // norm!(success_unit_IfAlternativesIdentical, "unit/IfAlternativesIdentical");
    norm!(success_unit_IfFalse, "unit/IfFalse");
    norm!(success_unit_IfNormalizePredicateAndBranches, "unit/IfNormalizePredicateAndBranches");
    // norm!(success_unit_IfTrivial, "unit/IfTrivial");
    norm!(success_unit_IfTrue, "unit/IfTrue");
    norm!(success_unit_Integer, "unit/Integer");
    norm!(success_unit_IntegerNegative, "unit/IntegerNegative");
    norm!(success_unit_IntegerPositive, "unit/IntegerPositive");
    // norm!(success_unit_IntegerShow_12, "unit/IntegerShow-12");
    // norm!(success_unit_IntegerShow12, "unit/IntegerShow12");
    norm!(success_unit_IntegerShow, "unit/IntegerShow");
    // norm!(success_unit_IntegerToDouble_12, "unit/IntegerToDouble-12");
    // norm!(success_unit_IntegerToDouble12, "unit/IntegerToDouble12");
    norm!(success_unit_IntegerToDouble, "unit/IntegerToDouble");
    norm!(success_unit_Kind, "unit/Kind");
    norm!(success_unit_Let, "unit/Let");
    norm!(success_unit_LetWithType, "unit/LetWithType");
    norm!(success_unit_List, "unit/List");
    norm!(success_unit_ListBuild, "unit/ListBuild");
    norm!(success_unit_ListBuildFoldFusion, "unit/ListBuildFoldFusion");
    norm!(success_unit_ListBuildImplementation, "unit/ListBuildImplementation");
    norm!(success_unit_ListFold, "unit/ListFold");
    norm!(success_unit_ListFoldEmpty, "unit/ListFoldEmpty");
    norm!(success_unit_ListFoldOne, "unit/ListFoldOne");
    norm!(success_unit_ListHead, "unit/ListHead");
    norm!(success_unit_ListHeadEmpty, "unit/ListHeadEmpty");
    norm!(success_unit_ListHeadOne, "unit/ListHeadOne");
    norm!(success_unit_ListIndexed, "unit/ListIndexed");
    norm!(success_unit_ListIndexedEmpty, "unit/ListIndexedEmpty");
    norm!(success_unit_ListIndexedOne, "unit/ListIndexedOne");
    norm!(success_unit_ListLast, "unit/ListLast");
    norm!(success_unit_ListLastEmpty, "unit/ListLastEmpty");
    norm!(success_unit_ListLastOne, "unit/ListLastOne");
    norm!(success_unit_ListLength, "unit/ListLength");
    norm!(success_unit_ListLengthEmpty, "unit/ListLengthEmpty");
    norm!(success_unit_ListLengthOne, "unit/ListLengthOne");
    norm!(success_unit_ListNormalizeElements, "unit/ListNormalizeElements");
    norm!(success_unit_ListNormalizeTypeAnnotation, "unit/ListNormalizeTypeAnnotation");
    norm!(success_unit_ListReverse, "unit/ListReverse");
    norm!(success_unit_ListReverseEmpty, "unit/ListReverseEmpty");
    norm!(success_unit_ListReverseTwo, "unit/ListReverseTwo");
    norm!(success_unit_Merge, "unit/Merge");
    norm!(success_unit_MergeEmptyAlternative, "unit/MergeEmptyAlternative");
    norm!(success_unit_MergeNormalizeArguments, "unit/MergeNormalizeArguments");
    norm!(success_unit_MergeWithType, "unit/MergeWithType");
    norm!(success_unit_MergeWithTypeNormalizeArguments, "unit/MergeWithTypeNormalizeArguments");
    norm!(success_unit_Natural, "unit/Natural");
    norm!(success_unit_NaturalBuild, "unit/NaturalBuild");
    norm!(success_unit_NaturalBuildFoldFusion, "unit/NaturalBuildFoldFusion");
    norm!(success_unit_NaturalBuildImplementation, "unit/NaturalBuildImplementation");
    norm!(success_unit_NaturalEven, "unit/NaturalEven");
    norm!(success_unit_NaturalEvenOne, "unit/NaturalEvenOne");
    norm!(success_unit_NaturalEvenZero, "unit/NaturalEvenZero");
    norm!(success_unit_NaturalFold, "unit/NaturalFold");
    norm!(success_unit_NaturalFoldOne, "unit/NaturalFoldOne");
    norm!(success_unit_NaturalFoldZero, "unit/NaturalFoldZero");
    norm!(success_unit_NaturalIsZero, "unit/NaturalIsZero");
    norm!(success_unit_NaturalIsZeroOne, "unit/NaturalIsZeroOne");
    norm!(success_unit_NaturalIsZeroZero, "unit/NaturalIsZeroZero");
    norm!(success_unit_NaturalLiteral, "unit/NaturalLiteral");
    norm!(success_unit_NaturalOdd, "unit/NaturalOdd");
    norm!(success_unit_NaturalOddOne, "unit/NaturalOddOne");
    norm!(success_unit_NaturalOddZero, "unit/NaturalOddZero");
    norm!(success_unit_NaturalShow, "unit/NaturalShow");
    norm!(success_unit_NaturalShowOne, "unit/NaturalShowOne");
    norm!(success_unit_NaturalToInteger, "unit/NaturalToInteger");
    norm!(success_unit_NaturalToIntegerOne, "unit/NaturalToIntegerOne");
    norm!(success_unit_None, "unit/None");
    norm!(success_unit_NoneNatural, "unit/NoneNatural");
    // norm!(success_unit_OperatorAndEquivalentArguments, "unit/OperatorAndEquivalentArguments");
    norm!(success_unit_OperatorAndLhsFalse, "unit/OperatorAndLhsFalse");
    norm!(success_unit_OperatorAndLhsTrue, "unit/OperatorAndLhsTrue");
    norm!(success_unit_OperatorAndNormalizeArguments, "unit/OperatorAndNormalizeArguments");
    norm!(success_unit_OperatorAndRhsFalse, "unit/OperatorAndRhsFalse");
    norm!(success_unit_OperatorAndRhsTrue, "unit/OperatorAndRhsTrue");
    // norm!(success_unit_OperatorEqualEquivalentArguments, "unit/OperatorEqualEquivalentArguments");
    norm!(success_unit_OperatorEqualLhsTrue, "unit/OperatorEqualLhsTrue");
    norm!(success_unit_OperatorEqualNormalizeArguments, "unit/OperatorEqualNormalizeArguments");
    norm!(success_unit_OperatorEqualRhsTrue, "unit/OperatorEqualRhsTrue");
    norm!(success_unit_OperatorListConcatenateLhsEmpty, "unit/OperatorListConcatenateLhsEmpty");
    norm!(success_unit_OperatorListConcatenateListList, "unit/OperatorListConcatenateListList");
    norm!(success_unit_OperatorListConcatenateNormalizeArguments, "unit/OperatorListConcatenateNormalizeArguments");
    norm!(success_unit_OperatorListConcatenateRhsEmpty, "unit/OperatorListConcatenateRhsEmpty");
    // norm!(success_unit_OperatorNotEqualEquivalentArguments, "unit/OperatorNotEqualEquivalentArguments");
    norm!(success_unit_OperatorNotEqualLhsFalse, "unit/OperatorNotEqualLhsFalse");
    norm!(success_unit_OperatorNotEqualNormalizeArguments, "unit/OperatorNotEqualNormalizeArguments");
    norm!(success_unit_OperatorNotEqualRhsFalse, "unit/OperatorNotEqualRhsFalse");
    // norm!(success_unit_OperatorOrEquivalentArguments, "unit/OperatorOrEquivalentArguments");
    norm!(success_unit_OperatorOrLhsFalse, "unit/OperatorOrLhsFalse");
    norm!(success_unit_OperatorOrLhsTrue, "unit/OperatorOrLhsTrue");
    norm!(success_unit_OperatorOrNormalizeArguments, "unit/OperatorOrNormalizeArguments");
    norm!(success_unit_OperatorOrRhsFalse, "unit/OperatorOrRhsFalse");
    norm!(success_unit_OperatorOrRhsTrue, "unit/OperatorOrRhsTrue");
    norm!(success_unit_OperatorPlusLhsZero, "unit/OperatorPlusLhsZero");
    norm!(success_unit_OperatorPlusNormalizeArguments, "unit/OperatorPlusNormalizeArguments");
    norm!(success_unit_OperatorPlusOneAndOne, "unit/OperatorPlusOneAndOne");
    norm!(success_unit_OperatorPlusRhsZero, "unit/OperatorPlusRhsZero");
    // norm!(success_unit_OperatorTextConcatenateLhsEmpty, "unit/OperatorTextConcatenateLhsEmpty");
    // norm!(success_unit_OperatorTextConcatenateNormalizeArguments, "unit/OperatorTextConcatenateNormalizeArguments");
    // norm!(success_unit_OperatorTextConcatenateRhsEmpty, "unit/OperatorTextConcatenateRhsEmpty");
    norm!(success_unit_OperatorTextConcatenateTextText, "unit/OperatorTextConcatenateTextText");
    norm!(success_unit_OperatorTimesLhsOne, "unit/OperatorTimesLhsOne");
    norm!(success_unit_OperatorTimesLhsZero, "unit/OperatorTimesLhsZero");
    norm!(success_unit_OperatorTimesNormalizeArguments, "unit/OperatorTimesNormalizeArguments");
    norm!(success_unit_OperatorTimesRhsOne, "unit/OperatorTimesRhsOne");
    norm!(success_unit_OperatorTimesRhsZero, "unit/OperatorTimesRhsZero");
    norm!(success_unit_OperatorTimesTwoAndTwo, "unit/OperatorTimesTwoAndTwo");
    norm!(success_unit_Optional, "unit/Optional");
    norm!(success_unit_OptionalBuild, "unit/OptionalBuild");
    norm!(success_unit_OptionalBuildFoldFusion, "unit/OptionalBuildFoldFusion");
    norm!(success_unit_OptionalBuildImplementation, "unit/OptionalBuildImplementation");
    norm!(success_unit_OptionalFold, "unit/OptionalFold");
    norm!(success_unit_OptionalFoldNone, "unit/OptionalFoldNone");
    norm!(success_unit_OptionalFoldSome, "unit/OptionalFoldSome");
    norm!(success_unit_Record, "unit/Record");
    norm!(success_unit_RecordEmpty, "unit/RecordEmpty");
    norm!(success_unit_RecordProjection, "unit/RecordProjection");
    norm!(success_unit_RecordProjectionEmpty, "unit/RecordProjectionEmpty");
    norm!(success_unit_RecordProjectionNormalizeArguments, "unit/RecordProjectionNormalizeArguments");
    norm!(success_unit_RecordSelection, "unit/RecordSelection");
    norm!(success_unit_RecordSelectionNormalizeArguments, "unit/RecordSelectionNormalizeArguments");
    norm!(success_unit_RecordType, "unit/RecordType");
    norm!(success_unit_RecordTypeEmpty, "unit/RecordTypeEmpty");
    // norm!(success_unit_RecursiveRecordMergeCollision, "unit/RecursiveRecordMergeCollision");
    // norm!(success_unit_RecursiveRecordMergeLhsEmpty, "unit/RecursiveRecordMergeLhsEmpty");
    // norm!(success_unit_RecursiveRecordMergeNoCollision, "unit/RecursiveRecordMergeNoCollision");
    // norm!(success_unit_RecursiveRecordMergeNormalizeArguments, "unit/RecursiveRecordMergeNormalizeArguments");
    // norm!(success_unit_RecursiveRecordMergeRhsEmpty, "unit/RecursiveRecordMergeRhsEmpty");
    // norm!(success_unit_RecursiveRecordTypeMergeCollision, "unit/RecursiveRecordTypeMergeCollision");
    // norm!(success_unit_RecursiveRecordTypeMergeLhsEmpty, "unit/RecursiveRecordTypeMergeLhsEmpty");
    // norm!(success_unit_RecursiveRecordTypeMergeNoCollision, "unit/RecursiveRecordTypeMergeNoCollision");
    // norm!(success_unit_RecursiveRecordTypeMergeNormalizeArguments, "unit/RecursiveRecordTypeMergeNormalizeArguments");
    // norm!(success_unit_RecursiveRecordTypeMergeRhsEmpty, "unit/RecursiveRecordTypeMergeRhsEmpty");
    // norm!(success_unit_RecursiveRecordTypeMergeSorts, "unit/RecursiveRecordTypeMergeSorts");
    // norm!(success_unit_RightBiasedRecordMergeCollision, "unit/RightBiasedRecordMergeCollision");
    // norm!(success_unit_RightBiasedRecordMergeLhsEmpty, "unit/RightBiasedRecordMergeLhsEmpty");
    // norm!(success_unit_RightBiasedRecordMergeNoCollision, "unit/RightBiasedRecordMergeNoCollision");
    // norm!(success_unit_RightBiasedRecordMergeNormalizeArguments, "unit/RightBiasedRecordMergeNormalizeArguments");
    // norm!(success_unit_RightBiasedRecordMergeRhsEmpty, "unit/RightBiasedRecordMergeRhsEmpty");
    norm!(success_unit_SomeNormalizeArguments, "unit/SomeNormalizeArguments");
    norm!(success_unit_Sort, "unit/Sort");
    norm!(success_unit_Text, "unit/Text");
    norm!(success_unit_TextInterpolate, "unit/TextInterpolate");
    norm!(success_unit_TextLiteral, "unit/TextLiteral");
    norm!(success_unit_TextNormalizeInterpolations, "unit/TextNormalizeInterpolations");
    norm!(success_unit_TextShow, "unit/TextShow");
    // norm!(success_unit_TextShowAllEscapes, "unit/TextShowAllEscapes");
    norm!(success_unit_True, "unit/True");
    norm!(success_unit_Type, "unit/Type");
    norm!(success_unit_TypeAnnotation, "unit/TypeAnnotation");
    norm!(success_unit_UnionNormalizeAlternatives, "unit/UnionNormalizeAlternatives");
    norm!(success_unit_UnionNormalizeArguments, "unit/UnionNormalizeArguments");
    norm!(success_unit_UnionProjectConstructor, "unit/UnionProjectConstructor");
    norm!(success_unit_UnionSortAlternatives, "unit/UnionSortAlternatives");
    norm!(success_unit_UnionType, "unit/UnionType");
    norm!(success_unit_UnionTypeEmpty, "unit/UnionTypeEmpty");
    norm!(success_unit_UnionTypeNormalizeArguments, "unit/UnionTypeNormalizeArguments");
    norm!(success_unit_Variable, "unit/Variable");
}
