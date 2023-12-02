use ast::Type;

use super::array::array;
use super::blob::blob;
use super::function::fun;
use super::matrix::matrix;
use super::noop;
use super::path::path;
use super::rust;
use super::t;
use super::tc;
use super::tuple::tuple;
use super::u32::u32;
use super::unit::unit;

pub(crate) fn image() -> Type {
    tc("Image", [])
}

pub(crate) fn declare(builder: &mut super::Builder) {
    builder
        .t("Image", [], [rust("Image")])
        .f("load_image", [], [blob()], image(), [rust("Image::new")])
        .f("crop", [], [image(), u32(), u32(), u32(), u32()], image(), [rust("Image::crop")])
        .f("center_crop", [], [image(), u32(), u32()], image(), [rust("Image::center_crop")])
        .f("resize", [], [image(), u32(), u32()], image(), [rust("Image::resize")])
        .f("resize_width", [], [image(), u32()], image(), [rust("Image::resize_width")])
        .f("resize_height", [], [image(), u32()], image(), [rust("Image::height")])
        .f("into_matrix", [], [image()], matrix(t("f32")), [rust("Image::into_matrix")])
        .f("from_matrix", [], [matrix(t("f32"))], image(), [rust("Image::from_matrix")])
        .f("save", [], [image(), path()], unit(), [rust("Image::save")])
        .f("height", [], [image()], u32(), [rust("Image::height")])
        .f("width", [], [image()], u32(), [rust("Image::width")])
        .f("draw_box", [], [image(), u32(), u32(), u32(), u32(), array(t("u8"), Some(4))], image(), [rust("Image::draw_box")])
        .f("preview", [], [image()], unit(), [rust(noop())]);
}
