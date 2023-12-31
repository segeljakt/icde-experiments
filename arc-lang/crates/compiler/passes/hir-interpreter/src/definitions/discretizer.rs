use std::io::Result;
use std::io::Write;

use builtins::assigner::Assigner;
use hir::Type;

use super::Value;

pub fn define(builder: &mut super::Bifs) {
    builder
        .f("tumbling", |ctx, t, v| {
            let a0 = v[0].as_duration();
            Assigner::tumbling(a0).into()
        })
        .f("sliding", |ctx, t, v| {
            let a0 = v[0].as_duration();
            let a1 = v[1].as_duration();
            Assigner::sliding(a0, a1).into()
        })
        .f("session", |ctx, t, v| {
            let a0 = v[0].as_duration();
            Assigner::session(a0).into()
        })
        .f("counting", |ctx, t, v| {
            let a0 = v[0].as_i32();
            Assigner::counting(a0).into()
        })
        .f("moving", |ctx, t, v| {
            let a0 = v[0].as_i32();
            let a1 = v[1].as_i32();
            Assigner::moving(a0, a1).into()
        });
}
