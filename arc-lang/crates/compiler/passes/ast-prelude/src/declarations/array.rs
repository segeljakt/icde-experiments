use std::io::Result;
use std::io::Write;

use super::rust;
use super::tuple::tuple;
use super::usize::usize;
use super::vec::vec;
use ast::Type;
use ast::TypeKind::TArray;
use im_rc::vector;
use im_rc::Vector;
use info::Info;

use super::t;

pub(crate) fn array(t: Type, n: Option<i32>) -> Type {
    TArray(t, n).with(Info::Builtin)
}

pub(crate) fn declare(builder: &mut super::Builder) {
    builder
        .f("array_get", ["T"], [array(t("T"), None), usize()], t("T"), [rust("Array::get")])
        .f("array_set", ["T"], [array(t("T"), None), usize(), t("T")], array(t("T"), None), [rust("Array::set")])
        .f("array_into_vec", ["T"], [array(t("T"), None)], vec(t("T")), [rust("Array::into_vec")])
        .f("array_into_set", ["T"], [array(t("T"), None)], vec(t("T")), [rust("Array::into_set")])
        .f("array_into_dict", ["T", "U"], [array(tuple([t("T"), t("U")]), None)], vec(tuple([t("T"), t("U")])), [rust("Array::into_dict")])
        .f("array_iter", ["T"], [array(t("T"), None)], vec(t("T")), [rust("Array::iter")]);
}
