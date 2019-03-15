#![feature(custom_inner_attributes)]
#![rustfmt::skip]
mod common;

macro_rules! tc_success {
    ($name:ident, $path:expr) => {
        make_spec_test!(TypecheckSuccess, $name, $path);
    };
}
macro_rules! tc_failure {
    ($name:ident, $path:expr) => {
        make_spec_test!(TypecheckFailure, $name, $path);
    };
}

// tc_success!(spec_typecheck_success_accessEncodedType, "accessEncodedType");
// tc_success!(spec_typecheck_success_accessType, "accessType");
tc_success!(spec_typecheck_success_prelude_Bool_and_0, "prelude/Bool/and/0");
tc_success!(spec_typecheck_success_prelude_Bool_and_1, "prelude/Bool/and/1");
tc_success!(spec_typecheck_success_prelude_Bool_build_0, "prelude/Bool/build/0");
tc_success!(spec_typecheck_success_prelude_Bool_build_1, "prelude/Bool/build/1");
tc_success!(spec_typecheck_success_prelude_Bool_even_0, "prelude/Bool/even/0");
tc_success!(spec_typecheck_success_prelude_Bool_even_1, "prelude/Bool/even/1");
tc_success!(spec_typecheck_success_prelude_Bool_even_2, "prelude/Bool/even/2");
tc_success!(spec_typecheck_success_prelude_Bool_even_3, "prelude/Bool/even/3");
tc_success!(spec_typecheck_success_prelude_Bool_fold_0, "prelude/Bool/fold/0");
tc_success!(spec_typecheck_success_prelude_Bool_fold_1, "prelude/Bool/fold/1");
tc_success!(spec_typecheck_success_prelude_Bool_not_0, "prelude/Bool/not/0");
tc_success!(spec_typecheck_success_prelude_Bool_not_1, "prelude/Bool/not/1");
tc_success!(spec_typecheck_success_prelude_Bool_odd_0, "prelude/Bool/odd/0");
tc_success!(spec_typecheck_success_prelude_Bool_odd_1, "prelude/Bool/odd/1");
tc_success!(spec_typecheck_success_prelude_Bool_odd_2, "prelude/Bool/odd/2");
tc_success!(spec_typecheck_success_prelude_Bool_odd_3, "prelude/Bool/odd/3");
tc_success!(spec_typecheck_success_prelude_Bool_or_0, "prelude/Bool/or/0");
tc_success!(spec_typecheck_success_prelude_Bool_or_1, "prelude/Bool/or/1");
tc_success!(spec_typecheck_success_prelude_Bool_show_0, "prelude/Bool/show/0");
tc_success!(spec_typecheck_success_prelude_Bool_show_1, "prelude/Bool/show/1");
// tc_success!(spec_typecheck_success_prelude_Double_show_0, "prelude/Double/show/0");
// tc_success!(spec_typecheck_success_prelude_Double_show_1, "prelude/Double/show/1");
// tc_success!(spec_typecheck_success_prelude_Integer_show_0, "prelude/Integer/show/0");
// tc_success!(spec_typecheck_success_prelude_Integer_show_1, "prelude/Integer/show/1");
// tc_success!(spec_typecheck_success_prelude_Integer_toDouble_0, "prelude/Integer/toDouble/0");
// tc_success!(spec_typecheck_success_prelude_Integer_toDouble_1, "prelude/Integer/toDouble/1");
tc_success!(spec_typecheck_success_prelude_List_all_0, "prelude/List/all/0");
tc_success!(spec_typecheck_success_prelude_List_all_1, "prelude/List/all/1");
tc_success!(spec_typecheck_success_prelude_List_any_0, "prelude/List/any/0");
tc_success!(spec_typecheck_success_prelude_List_any_1, "prelude/List/any/1");
tc_success!(spec_typecheck_success_prelude_List_build_0, "prelude/List/build/0");
tc_success!(spec_typecheck_success_prelude_List_build_1, "prelude/List/build/1");
tc_success!(spec_typecheck_success_prelude_List_concat_0, "prelude/List/concat/0");
tc_success!(spec_typecheck_success_prelude_List_concat_1, "prelude/List/concat/1");
tc_success!(spec_typecheck_success_prelude_List_concatMap_0, "prelude/List/concatMap/0");
tc_success!(spec_typecheck_success_prelude_List_concatMap_1, "prelude/List/concatMap/1");
tc_success!(spec_typecheck_success_prelude_List_filter_0, "prelude/List/filter/0");
tc_success!(spec_typecheck_success_prelude_List_filter_1, "prelude/List/filter/1");
tc_success!(spec_typecheck_success_prelude_List_fold_0, "prelude/List/fold/0");
tc_success!(spec_typecheck_success_prelude_List_fold_1, "prelude/List/fold/1");
tc_success!(spec_typecheck_success_prelude_List_fold_2, "prelude/List/fold/2");
tc_success!(spec_typecheck_success_prelude_List_generate_0, "prelude/List/generate/0");
tc_success!(spec_typecheck_success_prelude_List_generate_1, "prelude/List/generate/1");
tc_success!(spec_typecheck_success_prelude_List_head_0, "prelude/List/head/0");
tc_success!(spec_typecheck_success_prelude_List_head_1, "prelude/List/head/1");
tc_success!(spec_typecheck_success_prelude_List_indexed_0, "prelude/List/indexed/0");
tc_success!(spec_typecheck_success_prelude_List_indexed_1, "prelude/List/indexed/1");
tc_success!(spec_typecheck_success_prelude_List_iterate_0, "prelude/List/iterate/0");
tc_success!(spec_typecheck_success_prelude_List_iterate_1, "prelude/List/iterate/1");
tc_success!(spec_typecheck_success_prelude_List_last_0, "prelude/List/last/0");
tc_success!(spec_typecheck_success_prelude_List_last_1, "prelude/List/last/1");
tc_success!(spec_typecheck_success_prelude_List_length_0, "prelude/List/length/0");
tc_success!(spec_typecheck_success_prelude_List_length_1, "prelude/List/length/1");
tc_success!(spec_typecheck_success_prelude_List_map_0, "prelude/List/map/0");
tc_success!(spec_typecheck_success_prelude_List_map_1, "prelude/List/map/1");
tc_success!(spec_typecheck_success_prelude_List_null_0, "prelude/List/null/0");
tc_success!(spec_typecheck_success_prelude_List_null_1, "prelude/List/null/1");
tc_success!(spec_typecheck_success_prelude_List_replicate_0, "prelude/List/replicate/0");
tc_success!(spec_typecheck_success_prelude_List_replicate_1, "prelude/List/replicate/1");
tc_success!(spec_typecheck_success_prelude_List_reverse_0, "prelude/List/reverse/0");
tc_success!(spec_typecheck_success_prelude_List_reverse_1, "prelude/List/reverse/1");
tc_success!(spec_typecheck_success_prelude_List_shifted_0, "prelude/List/shifted/0");
tc_success!(spec_typecheck_success_prelude_List_shifted_1, "prelude/List/shifted/1");
tc_success!(spec_typecheck_success_prelude_List_unzip_0, "prelude/List/unzip/0");
tc_success!(spec_typecheck_success_prelude_List_unzip_1, "prelude/List/unzip/1");
// tc_success!(spec_typecheck_success_prelude_Monoid_00, "prelude/Monoid/00");
// tc_success!(spec_typecheck_success_prelude_Monoid_01, "prelude/Monoid/01");
// tc_success!(spec_typecheck_success_prelude_Monoid_02, "prelude/Monoid/02");
// tc_success!(spec_typecheck_success_prelude_Monoid_03, "prelude/Monoid/03");
// tc_success!(spec_typecheck_success_prelude_Monoid_04, "prelude/Monoid/04");
// tc_success!(spec_typecheck_success_prelude_Monoid_05, "prelude/Monoid/05");
// tc_success!(spec_typecheck_success_prelude_Monoid_06, "prelude/Monoid/06");
// tc_success!(spec_typecheck_success_prelude_Monoid_07, "prelude/Monoid/07");
// tc_success!(spec_typecheck_success_prelude_Monoid_08, "prelude/Monoid/08");
// tc_success!(spec_typecheck_success_prelude_Monoid_09, "prelude/Monoid/09");
// tc_success!(spec_typecheck_success_prelude_Monoid_10, "prelude/Monoid/10");
tc_success!(spec_typecheck_success_prelude_Natural_build_0, "prelude/Natural/build/0");
tc_success!(spec_typecheck_success_prelude_Natural_build_1, "prelude/Natural/build/1");
tc_success!(spec_typecheck_success_prelude_Natural_enumerate_0, "prelude/Natural/enumerate/0");
tc_success!(spec_typecheck_success_prelude_Natural_enumerate_1, "prelude/Natural/enumerate/1");
tc_success!(spec_typecheck_success_prelude_Natural_even_0, "prelude/Natural/even/0");
tc_success!(spec_typecheck_success_prelude_Natural_even_1, "prelude/Natural/even/1");
tc_success!(spec_typecheck_success_prelude_Natural_fold_0, "prelude/Natural/fold/0");
tc_success!(spec_typecheck_success_prelude_Natural_fold_1, "prelude/Natural/fold/1");
tc_success!(spec_typecheck_success_prelude_Natural_fold_2, "prelude/Natural/fold/2");
tc_success!(spec_typecheck_success_prelude_Natural_isZero_0, "prelude/Natural/isZero/0");
tc_success!(spec_typecheck_success_prelude_Natural_isZero_1, "prelude/Natural/isZero/1");
tc_success!(spec_typecheck_success_prelude_Natural_odd_0, "prelude/Natural/odd/0");
tc_success!(spec_typecheck_success_prelude_Natural_odd_1, "prelude/Natural/odd/1");
tc_success!(spec_typecheck_success_prelude_Natural_product_0, "prelude/Natural/product/0");
tc_success!(spec_typecheck_success_prelude_Natural_product_1, "prelude/Natural/product/1");
// tc_success!(spec_typecheck_success_prelude_Natural_show_0, "prelude/Natural/show/0");
// tc_success!(spec_typecheck_success_prelude_Natural_show_1, "prelude/Natural/show/1");
tc_success!(spec_typecheck_success_prelude_Natural_sum_0, "prelude/Natural/sum/0");
tc_success!(spec_typecheck_success_prelude_Natural_sum_1, "prelude/Natural/sum/1");
// tc_success!(spec_typecheck_success_prelude_Natural_toDouble_0, "prelude/Natural/toDouble/0");
// tc_success!(spec_typecheck_success_prelude_Natural_toDouble_1, "prelude/Natural/toDouble/1");
// tc_success!(spec_typecheck_success_prelude_Natural_toInteger_0, "prelude/Natural/toInteger/0");
// tc_success!(spec_typecheck_success_prelude_Natural_toInteger_1, "prelude/Natural/toInteger/1");
// tc_success!(spec_typecheck_success_prelude_Optional_all_0, "prelude/Optional/all/0");
// tc_success!(spec_typecheck_success_prelude_Optional_all_1, "prelude/Optional/all/1");
// tc_success!(spec_typecheck_success_prelude_Optional_any_0, "prelude/Optional/any/0");
// tc_success!(spec_typecheck_success_prelude_Optional_any_1, "prelude/Optional/any/1");
// tc_success!(spec_typecheck_success_prelude_Optional_build_0, "prelude/Optional/build/0");
// tc_success!(spec_typecheck_success_prelude_Optional_build_1, "prelude/Optional/build/1");
// tc_success!(spec_typecheck_success_prelude_Optional_concat_0, "prelude/Optional/concat/0");
// tc_success!(spec_typecheck_success_prelude_Optional_concat_1, "prelude/Optional/concat/1");
// tc_success!(spec_typecheck_success_prelude_Optional_concat_2, "prelude/Optional/concat/2");
// tc_success!(spec_typecheck_success_prelude_Optional_filter_0, "prelude/Optional/filter/0");
// tc_success!(spec_typecheck_success_prelude_Optional_filter_1, "prelude/Optional/filter/1");
// tc_success!(spec_typecheck_success_prelude_Optional_fold_0, "prelude/Optional/fold/0");
// tc_success!(spec_typecheck_success_prelude_Optional_fold_1, "prelude/Optional/fold/1");
// tc_success!(spec_typecheck_success_prelude_Optional_head_0, "prelude/Optional/head/0");
// tc_success!(spec_typecheck_success_prelude_Optional_head_1, "prelude/Optional/head/1");
// tc_success!(spec_typecheck_success_prelude_Optional_head_2, "prelude/Optional/head/2");
// tc_success!(spec_typecheck_success_prelude_Optional_last_0, "prelude/Optional/last/0");
// tc_success!(spec_typecheck_success_prelude_Optional_last_1, "prelude/Optional/last/1");
// tc_success!(spec_typecheck_success_prelude_Optional_last_2, "prelude/Optional/last/2");
// tc_success!(spec_typecheck_success_prelude_Optional_length_0, "prelude/Optional/length/0");
// tc_success!(spec_typecheck_success_prelude_Optional_length_1, "prelude/Optional/length/1");
// tc_success!(spec_typecheck_success_prelude_Optional_map_0, "prelude/Optional/map/0");
// tc_success!(spec_typecheck_success_prelude_Optional_map_1, "prelude/Optional/map/1");
// tc_success!(spec_typecheck_success_prelude_Optional_null_0, "prelude/Optional/null/0");
// tc_success!(spec_typecheck_success_prelude_Optional_null_1, "prelude/Optional/null/1");
// tc_success!(spec_typecheck_success_prelude_Optional_toList_0, "prelude/Optional/toList/0");
// tc_success!(spec_typecheck_success_prelude_Optional_toList_1, "prelude/Optional/toList/1");
// tc_success!(spec_typecheck_success_prelude_Optional_unzip_0, "prelude/Optional/unzip/0");
// tc_success!(spec_typecheck_success_prelude_Optional_unzip_1, "prelude/Optional/unzip/1");
tc_success!(spec_typecheck_success_prelude_Text_concat_0, "prelude/Text/concat/0");
tc_success!(spec_typecheck_success_prelude_Text_concat_1, "prelude/Text/concat/1");
tc_success!(spec_typecheck_success_prelude_Text_concatMap_0, "prelude/Text/concatMap/0");
tc_success!(spec_typecheck_success_prelude_Text_concatMap_1, "prelude/Text/concatMap/1");
// tc_success!(spec_typecheck_success_prelude_Text_concatMapSep_0, "prelude/Text/concatMapSep/0");
// tc_success!(spec_typecheck_success_prelude_Text_concatMapSep_1, "prelude/Text/concatMapSep/1");
// tc_success!(spec_typecheck_success_prelude_Text_concatSep_0, "prelude/Text/concatSep/0");
// tc_success!(spec_typecheck_success_prelude_Text_concatSep_1, "prelude/Text/concatSep/1");
// tc_success!(spec_typecheck_success_recordOfRecordOfTypes, "recordOfRecordOfTypes");
// tc_success!(spec_typecheck_success_recordOfTypes, "recordOfTypes");
// tc_success!(spec_typecheck_success_simple_access_0, "simple/access/0");
// tc_success!(spec_typecheck_success_simple_access_1, "simple/access/1");
// tc_success!(spec_typecheck_success_simple_alternativesAreTypes, "simple/alternativesAreTypes");
// tc_success!(spec_typecheck_success_simple_anonymousFunctionsInTypes, "simple/anonymousFunctionsInTypes");
// tc_success!(spec_typecheck_success_simple_fieldsAreTypes, "simple/fieldsAreTypes");
// tc_success!(spec_typecheck_success_simple_kindParameter, "simple/kindParameter");
// tc_success!(spec_typecheck_success_simple_mergeEquivalence, "simple/mergeEquivalence");
// tc_success!(spec_typecheck_success_simple_mixedFieldAccess, "simple/mixedFieldAccess");
// tc_success!(spec_typecheck_success_simple_unionsOfTypes, "simple/unionsOfTypes");

// tc_failure!(spec_typecheck_failure_combineMixedRecords, "combineMixedRecords");
// tc_failure!(spec_typecheck_failure_duplicateFields, "duplicateFields");
tc_failure!(spec_typecheck_failure_hurkensParadox, "hurkensParadox");