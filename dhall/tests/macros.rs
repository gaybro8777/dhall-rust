#[macro_export]
macro_rules! include_test_str {
    ($x:expr) => { include_str!(concat!("../../dhall-lang/tests/", $x, ".dhall")) };
}

#[macro_export]
macro_rules! include_test_strs_ab {
    ($x:expr) => { (include_test_str!(concat!($x, "A")), include_test_str!(concat!($x, "B"))) };
}

#[macro_export]
macro_rules! run_spec_test {
    (normalization, $path:expr) => {
        // let (expr_str, expected_str) = include_test_strs_ab!($path);
        // let expr = parser::parse_expr(&expr_str).unwrap();
        // let expected = parser::parse_expr(&expected_str).unwrap();
        // assert_eq!(normalize::<_, X, _>(&expr), normalize::<_, X, _>(&expected));
    };
    (parser, $path:expr) => {
        let expr_str = include_test_str!(concat!($path, "A"));
        let pest_expr = parser::parse_expr_pest(&expr_str).map_err(|e| println!("{}", e)).unwrap();
        match parser::parse_expr_lalrpop(&expr_str) {
            Ok(larlpop_expr) => assert_eq!(pest_expr, larlpop_expr),
            Err(_) => {},
        };
    };
    (parser_failure, $path:expr) => {
        let expr_str = include_test_str!($path);
        parser::parse_expr_pest(&expr_str).unwrap_err();
    };
}

#[macro_export]
macro_rules! make_spec_test {
    ($type:ident, $name:ident, $path:expr) => {
        #[test]
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[allow(unused_imports)]
        fn $name(){
            use std::thread;
            use dhall::*;

            thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(move || {
                run_spec_test!($type, $path);
            }).unwrap().join().unwrap();
        }
    };
}