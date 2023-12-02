use std::io::Result;
use std::io::Write;

use ast::TypeKind::TArray;
use hir::Type;
use im_rc::vector;
use im_rc::Vector;
use serde::Deserialize;
use serde::Serialize;

use super::Value;
use builtins::dict::Dict;
use builtins::set::Set;
use builtins::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct Array(pub Vector<Value>);

pub fn define(builder: &mut super::Bifs) {
    builder
        .f("array_get", |ctx, t, v| {
            let v0 = v[0].as_array();
            let v1 = v[1].as_usize();
            v0.0[v1].clone()
        })
        .f("array_set", |ctx, t, v| {
            let mut v0 = v[0].as_array();
            let v1 = v[1].as_usize();
            let v2 = v[2].clone();
            v0.0[v1] = v2;
            v0.into()
        })
        .f("array_into_vec", |ctx, t, v| {
            let v0 = v[0].as_array();
            Vec::from(v0.0.into_iter().collect::<std::vec::Vec<_>>()).into()
        })
        .f("array_into_set", |ctx, t, v| {
            let v0 = v[0].as_array();
            Set::from(v0.0.into_iter().collect::<std::collections::HashSet<_>>()).into()
        })
        .f("array_into_dict", |ctx, t, v| {
            let v0 = v[0].as_array();
            Dict::from(
                v0.0.into_iter()
                    .map(|v| {
                        let v = v.as_tuple();
                        (v.0[0].clone(), v.0[1].clone())
                    })
                    .collect::<std::collections::HashMap<_, _>>(),
            )
            .into()
        });
}
