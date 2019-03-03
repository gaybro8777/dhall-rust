use std::collections::BTreeMap;
use itertools::*;
use lalrpop_util;
use pest::Parser;
use pest::iterators::Pair;

use dhall_parser::{DhallParser, Rule};

use crate::grammar;
use crate::grammar_util::{BoxExpr, ParsedExpr};
use crate::lexer::{Lexer, LexicalError, Tok};
use crate::core::{bx, Expr, Builtin, Const, V};

pub fn parse_expr_lalrpop(s: &str) -> Result<BoxExpr, lalrpop_util::ParseError<usize, Tok, LexicalError>>  {
    grammar::ExprParser::new().parse(Lexer::new(s))
}

pub type ParseError = pest::error::Error<Rule>;

pub type ParseResult<T> = Result<T, ParseError>;

pub fn custom_parse_error(pair: &Pair<Rule>, msg: String) -> ParseError {
    let e = pest::error::ErrorVariant::CustomError{ message: msg };
    pest::error::Error::new_from_span(e, pair.as_span())
}


/* Macro to pattern-match iterators.
 * Panics if the sequence doesn't match;
 *
 * Example:
 * ```
 * let vec = vec![1, 2, 3];
 *
 * match_iter!(vec.into_iter(); (x, y?, z) => {
 *  // x: T
 *  // y: Option<T>
 *  // z: T
 * })
 *
 * // or
 * match_iter!(vec.into_iter(); (x, y, z*) => {
 *  // x, y: T
 *  // z: impl Iterator<T>
 * })
 * ```
 *
 */
macro_rules! match_iter {
    // Everything else pattern
    (@match 0, $iter:expr, $x:ident* $($rest:tt)*) => {
        match_iter!(@match 2, $iter $($rest)*);
        #[allow(unused_mut)]
        let mut $x = $iter;
    };
    // Alias to use in macros
    (@match 0, $iter:expr, $x:ident?? $($rest:tt)*) => {
        match_iter!(@match 2, $iter $($rest)*);
        #[allow(unused_mut)]
        let mut $x = $iter;
    };
    // Optional pattern
    (@match 0, $iter:expr, $x:ident? $($rest:tt)*) => {
        match_iter!(@match 1, $iter $($rest)*);
        let $x = $iter.next();
        $iter.next().ok_or(()).expect_err("Some values remain unused");
    };
    // Normal pattern
    (@match 0, $iter:expr, $x:ident $($rest:tt)*) => {
        let $x = $iter.next().unwrap();
        match_iter!(@match 0, $iter $($rest)*);
    };
    // Normal pattern after a variable length one: declare reversed and take from the end
    (@match $w:expr, $iter:expr, $x:ident $($rest:tt)*) => {
        match_iter!(@match $w, $iter $($rest)*);
        let $x = $iter.next_back().unwrap();
    };

    // Check no elements remain
    (@match 0, $iter:expr) => {
        $iter.next().ok_or(()).expect_err("Some values remain unused");
    };
    (@match $_:expr, $iter:expr) => {};

    // Entrypoint
    ($iter:expr; ($($args:tt)*) => $body:expr) => {
        {
            #[allow(unused_mut)]
            let mut iter = $iter;
            match_iter!(@match 0, iter, $($args)*);
            $body
        }
    };
}



macro_rules! named {
    ($name:ident<$o:ty>; $submac:ident!( $($args:tt)* )) => (
        #[allow(unused_variables)]
        fn $name<'a>(pair: Pair<'a, Rule>) -> ParseResult<$o> {
            $submac!(pair; $($args)*)
        }
    );
}

macro_rules! match_children {
    (@collect, $pairs:expr, ($($args:tt)*), $body:expr, ($($acc:tt)*), ($x:ident : $ty:ident, $($rest:tt)*)) => {
        match_children!(@collect, $pairs, ($($args)*), $body, ($($acc)*, $x), ($($rest)*))
    };
    (@collect, $pairs:expr, ($($args:tt)*), $body:expr, ($($acc:tt)*), ($x:ident? : $ty:ident, $($rest:tt)*)) => {
        match_children!(@collect, $pairs, ($($args)*), $body, ($($acc)*, $x?), ($($rest)*))
    };
    (@collect, $pairs:expr, ($($args:tt)*), $body:expr, ($($acc:tt)*), ($x:ident* : $ty:ident, $($rest:tt)*)) => {
        match_children!(@collect, $pairs, ($($args)*), $body, ($($acc)*, $x??), ($($rest)*))
    };
    (@collect, $pairs:expr, ($($args:tt)*), $body:expr, (,$($acc:tt)*), ()) => {
        match_iter!($pairs; ($($acc)*) => {
            match_children!(@parse, $pairs, $($args)*);
            Ok($body)
        })
    };

    (@parse, $pairs:expr, $x:ident : $ty:ident $($rest:tt)*) => {
        let $x = $ty($x)?;
        match_children!(@parse, $pairs $($rest)*);
    };
    (@parse, $pairs:expr, $x:ident? : $ty:ident $($rest:tt)*) => {
        let $x = $x.map($ty).transpose()?;
        match_children!(@parse, $pairs $($rest)*);
    };
    (@parse, $pairs:expr, $x:ident* : $ty:ident $($rest:tt)*) => {
        #[allow(unused_mut)]
        let mut $x = $x.map($ty);
        match_children!(@parse, $pairs $($rest)*);
    };
    (@parse, $pairs:expr) => {};

    // Entrypoints
    ($pair:expr; $($rest:tt)*) => {
        {
            #[allow(unused_mut)]
            let mut pairs = $pair.into_inner();
            match_children!(@pairs; pairs; $($rest)*)
        }
    };
    (@pairs; $pairs:expr; ($($args:tt)*) => $body:expr) => {
        match_children!(@collect, $pairs, ($($args)*), $body, (), ($($args)*,))
    };
}

macro_rules! with_captured_str {
    ($pair:expr; $x:ident; $body:expr) => {
        {
            #[allow(unused_mut)]
            let mut $x = $pair.as_str();
            Ok($body)
        }
    };
}

macro_rules! with_raw_pair {
    ($pair:expr; $x:ident; $body:expr) => {
        {
            #[allow(unused_mut)]
            let mut $x = $pair;
            Ok($body)
        }
    };
}

macro_rules! map {
    ($pair:expr; $ty:ident; $f:expr) => {
        {
            let x = $ty($pair)?;
            Ok($f(x))
        }
    };
}

macro_rules! plain_value {
    ($_pair:expr; $body:expr) => {
        Ok($body)
    };
}

macro_rules! binop {
    ($pair:expr; $f:expr) => {
        {
            let f = $f;
            match_children!($pair; (first: expression, rest*: expression) => {
                rest.fold_results(first, |acc, e| bx(f(acc, e)))?
            })
        }
    };
}

macro_rules! with_rule {
    ($pair:expr; $x:ident; $submac:ident!( $($args:tt)* )) => {
        {
            #[allow(unused_mut)]
            let mut $x = $pair.as_rule();
            $submac!($pair; $($args)*)
        }
    };
}

macro_rules! match_rule {
    ($pair:expr; $($pat:pat => $submac:ident!( $($args:tt)* ),)*) => {
        {
            #[allow(unreachable_patterns)]
            match $pair.as_rule() {
                $(
                    $pat => $submac!($pair; $($args)*),
                )*
                r => Err(custom_parse_error(&$pair, format!("Unexpected {:?}", r))),
            }
        }
    };
}


named!(eoi<()>; plain_value!(()));

named!(str<&'a str>; with_captured_str!(s; { s.trim() }));

named!(natural<usize>; with_raw_pair!(pair; {
    pair.as_str().trim()
        .parse()
        .map_err(|e: std::num::ParseIntError| custom_parse_error(&pair, format!("{}", e)))?
}));

named!(integer<isize>; with_raw_pair!(pair; {
    pair.as_str().trim()
        .parse()
        .map_err(|e: std::num::ParseIntError| custom_parse_error(&pair, format!("{}", e)))?
}));

named!(letbinding<(&'a str, Option<BoxExpr<'a>>, BoxExpr<'a>)>;
    match_children!((name: str, annot?: expression, expr: expression) => (name, annot, expr))
);

named!(record_entry<(&'a str, BoxExpr<'a>)>;
    match_children!((name: str, expr: expression) => (name, expr))
);

named!(partial_record_entries<(Rule, BoxExpr<'a>, BTreeMap<&'a str, ParsedExpr<'a>>)>;
   with_rule!(rule;
        match_children!((expr: expression, entries*: record_entry) => {
            let mut map: BTreeMap<&str, ParsedExpr> = BTreeMap::new();
            for entry in entries {
                let (n, e) = entry?;
                map.insert(n, *e);
            }
            (rule, expr, map)
        })
    )
);

named!(expression<BoxExpr<'a>>; match_rule!(
    Rule::natural_literal_raw => map!(natural; |n| bx(Expr::NaturalLit(n))),
    Rule::integer_literal_raw => map!(integer; |n| bx(Expr::IntegerLit(n))),

    Rule::identifier_raw =>
        match_children!((name: str, idx?: natural) => {
            match Builtin::parse(name) {
                Some(b) => bx(Expr::Builtin(b)),
                None => match name {
                    "True" => bx(Expr::BoolLit(true)),
                    "False" => bx(Expr::BoolLit(false)),
                    "Type" => bx(Expr::Const(Const::Type)),
                    "Kind" => bx(Expr::Const(Const::Kind)),
                    name => bx(Expr::Var(V(name, idx.unwrap_or(0)))),
                }
            }
        }),

    Rule::lambda_expression =>
        match_children!((label: str, typ: expression, body: expression) => {
            bx(Expr::Lam(label, typ, body))
        }),

    Rule::ifthenelse_expression =>
        match_children!((cond: expression, left: expression, right: expression) => {
            bx(Expr::BoolIf(cond, left, right))
        }),

    Rule::let_expression =>
        match_children!((bindings*: letbinding, final_expr: expression) => {
            bindings.fold_results(final_expr, |acc, x| bx(Expr::Let(x.0, x.1, x.2, acc)))?
        }),

    Rule::forall_expression =>
        match_children!((label: str, typ: expression, body: expression) => {
            bx(Expr::Pi(label, typ, body))
        }),

    Rule::arrow_expression =>
        match_children!((typ: expression, body: expression) => {
            bx(Expr::Pi("_", typ, body))
        }),

    Rule::annotated_expression => binop!(Expr::Annot),
    Rule::import_alt_expression => binop!(Expr::ImportAlt),
    Rule::or_expression => binop!(Expr::BoolOr),
    Rule::plus_expression => binop!(Expr::NaturalPlus),
    Rule::text_append_expression => binop!(Expr::TextAppend),
    Rule::list_append_expression => binop!(Expr::ListAppend),
    Rule::and_expression => binop!(Expr::BoolAnd),
    Rule::combine_expression => binop!(Expr::Combine),
    Rule::prefer_expression => binop!(Expr::Prefer),
    Rule::combine_types_expression => binop!(Expr::CombineTypes),
    Rule::times_expression => binop!(Expr::NaturalTimes),
    Rule::equal_expression => binop!(Expr::BoolEQ),
    Rule::not_equal_expression => binop!(Expr::BoolNE),
    Rule::application_expression => binop!(Expr::App),

    Rule::selector_expression_raw =>
        match_children!((first: expression, rest*: str) => {
            rest.fold_results(first, |acc, e| bx(Expr::Field(acc, e)))?
        }),

    Rule::empty_record_type => plain_value!(bx(Expr::Record(BTreeMap::new()))),
    Rule::empty_record_literal => plain_value!(bx(Expr::RecordLit(BTreeMap::new()))),
    Rule::non_empty_record_type_or_literal =>
        match_children!((first_label: str, rest: partial_record_entries) => {
            let (rule, first_expr, mut map) = rest;
            map.insert(first_label, *first_expr);
            match rule {
                Rule::non_empty_record_type => bx(Expr::Record(map)),
                Rule::non_empty_record_literal => bx(Expr::RecordLit(map)),
                _ => unreachable!()
            }
        }),

    _ => with_rule!(rule;
        match_children!((exprs*: expression) => {
            // panic!();
            let rulename = format!("{:?}", rule);
            bx(Expr::FailedParse(rulename, exprs.map_results(|x| *x).collect::<ParseResult<_>>()?))
        })
    ),
));

named!(final_expression<BoxExpr<'a>>;
    match_children!((e: expression, _eoi: eoi) => e)
);


pub fn parse_expr_pest(s: &str) -> ParseResult<BoxExpr>  {
    let pairs = DhallParser::parse(Rule::final_expression, s)?;
    match_iter!(pairs; (e) => final_expression(e))
}


#[test]
fn test_parse() {
    use crate::core::Expr::*;
    // let expr = r#"{ x = "foo", y = 4 }.x"#;
    // let expr = r#"(1 + 2) * 3"#;
    let expr = r#"if True then 1 + 3 * 5 else 2"#;
    println!("{:?}", parse_expr_lalrpop(expr));
    match parse_expr_pest(expr) {
        Err(e) => {
            println!("{:?}", e);
            println!("{}", e);
        },
        ok => println!("{:?}", ok),
    }
    // assert_eq!(parse_expr_pest(expr).unwrap(), parse_expr_lalrpop(expr).unwrap());
    // assert!(false);

    println!("test {:?}", parse_expr_lalrpop("3 + 5 * 10"));
    assert!(parse_expr_lalrpop("22").is_ok());
    assert!(parse_expr_lalrpop("(22)").is_ok());
    assert_eq!(parse_expr_lalrpop("3 + 5 * 10").ok(),
               Some(Box::new(NaturalPlus(Box::new(NaturalLit(3)),
                                Box::new(NaturalTimes(Box::new(NaturalLit(5)),
                                                      Box::new(NaturalLit(10))))))));
    // The original parser is apparently right-associative
    assert_eq!(parse_expr_lalrpop("2 * 3 * 4").ok(),
               Some(Box::new(NaturalTimes(Box::new(NaturalLit(2)),
                                 Box::new(NaturalTimes(Box::new(NaturalLit(3)),
                                                       Box::new(NaturalLit(4))))))));
    assert!(parse_expr_lalrpop("((((22))))").is_ok());
    assert!(parse_expr_lalrpop("((22)").is_err());
    println!("{:?}", parse_expr_lalrpop("\\(b : Bool) -> b == False"));
    assert!(parse_expr_lalrpop("\\(b : Bool) -> b == False").is_ok());
    println!("{:?}", parse_expr_lalrpop("foo.bar"));
    assert!(parse_expr_lalrpop("foo.bar").is_ok());
    assert!(parse_expr_lalrpop("[] : List Bool").is_ok());

    // println!("{:?}", parse_expr_lalrpop("< Left = True | Right : Natural >"));
    // println!("{:?}", parse_expr_lalrpop(r#""bl${42}ah""#));
    // assert!(parse_expr_lalrpop("< Left = True | Right : Natural >").is_ok());
}