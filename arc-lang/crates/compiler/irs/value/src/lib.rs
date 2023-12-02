#![allow(unused)]

pub mod conv;
pub mod de;
pub mod dynamic;
pub mod eq;
pub mod hash;
pub mod ord;
pub mod ser;

use std::cell::RefCell;
use std::rc::Rc;

use builtins::aggregator::Aggregator;
use builtins::blob::Blob;
use builtins::dict::Dict;
use builtins::assigner::Assigner;
use builtins::duration::Duration;
use builtins::encoding::Encoding;
use builtins::file::File;
use builtins::image::Image;
use builtins::model::Model;
use builtins::path::Path;
use builtins::reader::Reader;
use builtins::set::Set;
use builtins::socket::SocketAddr;
use builtins::time::Time;
use builtins::time_source::TimeSource;
use builtins::url::Url;
use builtins::writer::Writer;
use dynamic::Array;
use dynamic::Dataflow;
use dynamic::Function;
use dynamic::Instance;
use dynamic::Matrix;
use dynamic::Record;
use dynamic::Stream;
use dynamic::Tuple;
use dynamic::Variant;
use hir::Name;
use im_rc::HashMap;
use im_rc::Vector;
use serde::Deserialize;
use serde::Serialize;
pub use ValueKind::*;

#[derive(Clone)]
pub struct Value {
    pub kind: Rc<ValueKind>,
}

impl From<ValueKind> for Value {
    fn from(kind: ValueKind) -> Self {
        Value::new(kind)
    }
}

impl Value {
    pub fn new(kind: ValueKind) -> Value {
        Value { kind: Rc::new(kind) }
    }
}

#[derive(Clone)]
pub enum ValueKind {
    VAggregator(Aggregator<Function, Function, Function, Function>),
    VArray(Array),
    VBlob(Blob),
    VBool(bool),
    VChar(char),
    VDict(Dict<Value, Value>),
    VDiscretizer(Assigner),
    VDuration(Duration),
    VEncoding(Encoding),
    VF32(f32),
    VF64(f64),
    VFile(File),
    VFunction(Function),
    VI128(i128),
    VI16(i16),
    VI32(i32),
    VI64(i64),
    VI8(i8),
    VImage(Image),
    VMatrix(Matrix),
    VModel(Model),
    VOption(builtins::option::Option<Value>),
    VPath(Path),
    VReader(Reader),
    VRecord(Record),
    VResult(builtins::result::Result<Value>),
    VSet(Set<Value>),
    VSocketAddr(SocketAddr),
    VStream(Stream),
    VDataflow(Dataflow),
    VString(builtins::string::String),
    VTime(Time),
    VTimeSource(TimeSource<Function>),
    VTuple(Tuple),
    VU128(u128),
    VU16(u16),
    VU32(u32),
    VU64(u64),
    VU8(u8),
    VUnit(()),
    VUrl(Url),
    VUsize(usize),
    VVariant(Variant),
    VVec(builtins::vec::Vec<Value>),
    VWriter(Writer),
    VInstance(Instance),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind.as_ref() {
            VAggregator(v) => v.fmt(f),
            VArray(v) => v.fmt(f),
            VBlob(v) => v.fmt(f),
            VBool(v) => v.fmt(f),
            VChar(v) => v.fmt(f),
            VDict(v) => v.fmt(f),
            VDiscretizer(v) => v.fmt(f),
            VDuration(v) => v.fmt(f),
            VEncoding(v) => v.fmt(f),
            VF32(v) => v.fmt(f),
            VF64(v) => v.fmt(f),
            VFile(v) => v.fmt(f),
            VFunction(v) => v.fmt(f),
            VI128(v) => v.fmt(f),
            VI16(v) => v.fmt(f),
            VI32(v) => v.fmt(f),
            VI64(v) => v.fmt(f),
            VI8(v) => v.fmt(f),
            VMatrix(v) => v.fmt(f),
            VModel(v) => v.fmt(f),
            VOption(v) => v.fmt(f),
            VPath(v) => v.fmt(f),
            VReader(v) => v.fmt(f),
            VRecord(v) => v.fmt(f),
            VResult(v) => v.fmt(f),
            VSet(v) => v.fmt(f),
            VSocketAddr(v) => v.fmt(f),
            VStream(v) => v.fmt(f),
            VDataflow(v) => v.fmt(f),
            VString(v) => v.fmt(f),
            VTime(v) => v.fmt(f),
            VTimeSource(v) => v.fmt(f),
            VTuple(v) => v.fmt(f),
            VU128(v) => v.fmt(f),
            VU16(v) => v.fmt(f),
            VU32(v) => v.fmt(f),
            VU64(v) => v.fmt(f),
            VU8(v) => v.fmt(f),
            VUnit(v) => v.fmt(f),
            VUrl(v) => v.fmt(f),
            VUsize(v) => v.fmt(f),
            VVariant(v) => v.fmt(f),
            VVec(v) => v.fmt(f),
            VWriter(v) => v.fmt(f),
            VInstance(v) => v.fmt(f),
            VImage(v) => v.fmt(f),
        }
    }
}
