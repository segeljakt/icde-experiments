use ast::Type;

use super::function::fun;
use super::rust;
use super::t;
use super::tc;
use super::tuple::tuple;

pub(crate) fn aggregator(i: Type, p: Type, o: Type) -> Type {
    tc("Aggregator", [i, p, o])
}

pub(crate) fn declare(builder: &mut super::Builder) {
    builder
        .t("Aggregator", ["I", "P", "O"], [rust("Aggregator")])
        //
        .f("aggregator", ["I", "P", "O"], [fun([t("I")], t("P")), fun([t("P"), t("P")], t("P")), fun([t("P")], t("O"))], aggregator(t("I"), t("P"), t("O")), [rust("Aggregator::aggregator")]);
}
