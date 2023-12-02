use std::io::Result;
use std::io::Write;
use std::rc::Rc;

use builtins::aggregator::Aggregator;
use hir::Name;
use hir::Type;
use serde::Deserialize;
use serde::Serialize;

use crate::definitions::*;

use super::*;
use value::dynamic::Function;

pub fn define(builder: &mut super::Bifs) {
    builder
        .f("incremental", |ctx, t, v| {
            let a0 = v[0].as_function();
            let a1 = v[1].as_function();
            let a2 = v[2].as_function();
            Aggregator::Incremental {
                lift: a0,
                combine: a1,
                lower: a2,
            }
            .into()
        })
        .f("holistic", |ctx, t, v| {
            let a0 = v[0].as_function();
            Aggregator::Holistic { compute: a0 }.into()
        });
}
