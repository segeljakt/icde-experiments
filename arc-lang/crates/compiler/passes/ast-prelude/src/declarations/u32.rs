use ast::Type;

use super::rust;
use super::t;

pub(crate) fn u32() -> Type {
    t("u32")
}

pub(crate) fn declare(builder: &mut super::Builder) {
    builder.t("u32", [], [rust("u32")]);
}
