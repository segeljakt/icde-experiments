use super::Value;
use builtins::vec::Vec;
use hir::TypeKind::TNominal;
use value::dynamic::Matrix;

use hir::Type;
use serde::Deserialize;
use serde::Serialize;
use value::map_matrix;
use value::map_matrix_inplace;

pub fn define(builder: &mut super::Bifs) {
    builder
        .f("zeros", |ctx, t, v| {
            let v0 = v[0]
                .as_array()
                .0
                .iter()
                .map(|x| x.as_usize())
                .collect::<std::vec::Vec<usize>>()
                .into();
            let TNominal(x, _) = t[0].kind.as_ref().clone() else {
                unreachable!()
            };
            let matrix: Matrix = match x.as_str() {
                "i8" => builtins::matrix::Matrix::<i8>::zeros(v0).into(),
                "i16" => builtins::matrix::Matrix::<i16>::zeros(v0).into(),
                "i32" => builtins::matrix::Matrix::<i32>::zeros(v0).into(),
                "i64" => builtins::matrix::Matrix::<i64>::zeros(v0).into(),
                "u8" => builtins::matrix::Matrix::<u8>::zeros(v0).into(),
                "u16" => builtins::matrix::Matrix::<u16>::zeros(v0).into(),
                "u32" => builtins::matrix::Matrix::<u32>::zeros(v0).into(),
                "u64" => builtins::matrix::Matrix::<u64>::zeros(v0).into(),
                "f32" => builtins::matrix::Matrix::<f32>::zeros(v0).into(),
                "f64" => builtins::matrix::Matrix::<f64>::zeros(v0).into(),
                _ => unreachable!(),
            };
            matrix.into()
        })
        .f("insert_axis", |ctx, t, v| {
            let v0 = v[0].as_matrix();
            let v1 = v[1].as_usize();
            map_matrix_inplace!(v0, |x: builtins::matrix::Matrix<_>| x.insert_axis(v1)).into()
        })
        .f("remove_axis", |ctx, t, v| {
            let v0 = v[0].as_matrix();
            let v1 = v[1].as_usize();
            map_matrix_inplace!(v0, |x: builtins::matrix::Matrix<_>| x.remove_axis(v1)).into()
        })
        .f("into_vec", |ctx, t, v| {
            let v0 = v[0].as_matrix();
            map_matrix!(v0, |x: builtins::matrix::Matrix<_>| {
                let v: builtins::vec::Vec<Value> = x
                    .into_vec()
                    .iter()
                    .map(|x| Value::from(x))
                    .collect_vec()
                    .into();
                v.into()
            })
        });
}
