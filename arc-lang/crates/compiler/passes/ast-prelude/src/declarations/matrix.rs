use ast::Type;

use super::array::array;
use super::rust;
use super::t;
use super::tc;
use super::usize::usize;
use super::vec::vec;

pub(crate) fn matrix(t: Type) -> Type {
    tc("Matrix", [t])
}

pub(crate) fn declare(builder: &mut super::Builder) {
    builder
        .t("Matrix", ["T"], [rust("Matrix")])
        .f("zeros", ["T"], [array(t("usize"), None)], matrix(t("T")), [])
        .f("insert_axis", ["T"], [matrix(t("T")), usize()], matrix(t("T")), [])
        .f("remove_axis", ["T"], [matrix(t("T")), usize()], matrix(t("T")), [])
        .f("into_vec", ["T"], [matrix(t("T"))], vec(t("T")), []);
}
