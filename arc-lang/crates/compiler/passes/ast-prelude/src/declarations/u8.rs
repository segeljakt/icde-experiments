use ast::Type;

use super::rust;
use super::t;

pub(crate) fn u8() -> Type {
    t("u8")
}

pub(crate) fn declare(builder: &mut super::Builder) {
    builder.t("u8", [], [rust("u8")]);
}
