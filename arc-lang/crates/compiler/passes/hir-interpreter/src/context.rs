#![allow(unused)]
use std::collections::HashMap;

use hir::*;
use im_rc::OrdMap;
use im_rc::Vector;
use info::Info;
use name_gen::NameGen;
use stack::Stack;

use value::Value;

use crate::definitions::Bifs;

#[derive(Debug)]
pub struct Context {
    pub(crate) stack: Stack<(), Value, ()>,
    pub(crate) funcs: HashMap<(Name, Vector<Type>), FuncDecl>,
    pub(crate) types: HashMap<(Name, Vector<Type>), TypeDecl>,
    pub(crate) bifs: Bifs,
    next_stream_name: NameGen,
    pub ss: Vector<hir::Stmt>,
    pub ctx7: hir_reachable::context::Context,
    pub ctx8: hir_to_mlir::context::Context,
    pub ctx9: hir_to_rust::context::Context,
    pub ctx10: build::context::Context,
    pub ctx11: kafka::context::Context,
}

#[derive(Clone, Debug)]
pub(crate) enum FuncDecl {
    Def(Meta, Vector<Pattern>, Type, Block),
    Bif(Meta, Vector<Type>, Type),
}

#[derive(Clone, Debug)]
pub(crate) enum TypeDecl {
    Enum(Vector<(Name, Type)>),
    Bit,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            stack: Stack::new(()),
            funcs: HashMap::new(),
            types: HashMap::new(),
            next_stream_name: NameGen::new("s"),
            bifs: Bifs::new(),
            ss: Vector::new(),
            ctx7: Default::default(),
            ctx8: Default::default(),
            ctx9: Default::default(),
            ctx10: Default::default(),
            ctx11: Default::default(),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Self::default()
    }

    pub fn find_val(&self, x: &Name) -> Value {
        if let Some(v) = self.stack.find_expr_decl(x) {
            v
        } else {
            panic!("{x} not found")
        }
    }

    pub fn new_stream_name(&mut self) -> Name {
        self.next_stream_name.fresh()
    }
}
