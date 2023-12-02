use std::io::Result;
use std::io::Write;
use std::rc::Rc;

use builtins::aggregator::Aggregator;
use builtins::array::Array;
use builtins::image::Image;
use hir::Name;
use hir::Type;
use serde::Deserialize;
use serde::Serialize;

use crate::definitions::*;

use super::*;
use value::dynamic::Function;

pub fn define(builder: &mut super::Bifs) {
    builder
        .f("load_image", |ctx, t, v| {
            let v0 = v[0].as_blob();
            Image::new(v0).into()
        })
        .f("crop", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_u32();
            let v2 = v[2].as_u32();
            let v3 = v[3].as_u32();
            let v4 = v[4].as_u32();
            v0.crop(v1, v2, v3, v4).into()
        })
        .f("center_crop", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_u32();
            let v2 = v[2].as_u32();
            v0.center_crop(v1, v2).into()
        })
        .f("resize", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_u32();
            let v2 = v[2].as_u32();
            v0.resize(v1, v2).into()
        })
        .f("resize_width", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_u32();
            v0.resize_width(v1).into()
        })
        .f("resize_height", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_u32();
            v0.resize_height(v1).into()
        })
        .f("into_matrix", |ctx, t, v| {
            let v0 = v[0].as_image();
            value::dynamic::Matrix::F32(v0.into_matrix()).into()
        })
        .f("from_matrix", |ctx, t, v| {
            let v0 = v[0].as_matrix();
            if let value::dynamic::Matrix::F32(v) = v0 {
                Image::from_matrix(v).into()
            } else {
                unreachable!()
            }
        })
        .f("save", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_path();
            v0.save(v1).into()
        })
        .f("height", |ctx, t, v| {
            let v0 = v[0].as_image();
            v0.height().into()
        })
        .f("width", |ctx, t, v| {
            let v0 = v[0].as_image();
            v0.width().into()
        })
        .f("draw_box", |ctx, t, v| {
            let v0 = v[0].as_image();
            let v1 = v[1].as_u32();
            let v2 = v[2].as_u32();
            let v3 = v[3].as_u32();
            let v4 = v[4].as_u32();
            let v5 = v[5]
                .as_array()
                .0
                .iter()
                .map(|v| v.as_u8())
                .collect::<Vec<_>>()
                .try_into()
                .map(|v: [u8; 4]| Array::from(v))
                .unwrap();
            v0.draw_box(v1, v2, v3, v4, v5.into()).into()
        })
        .f("preview", |ctx, t, v| {
            let v0 = v[0].as_image();
            let conf = viuer::Config {
                // set dimensions
                width: Some(80),
                height: Some(25),
                absolute_offset: false,
                use_kitty: true,
                use_iterm: true,
                ..Default::default()
            };
            viuer::print(&v0.0.as_ref().0, &conf).unwrap();
            ().into()
        });
}
