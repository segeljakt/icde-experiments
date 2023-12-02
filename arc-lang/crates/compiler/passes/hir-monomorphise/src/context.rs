use diagnostics::Diagnostics;
use hir::*;
use im_rc::HashMap;
use im_rc::HashSet;
use im_rc::OrdMap;
use im_rc::Vector;
use info::Info;
use stack::Stack;

#[derive(Debug, Clone)]
pub struct Context {
    pub(crate) stack: Stack<(), ExprDecl, TypeDecl>,
    pub monomorphised_types: HashSet<(Name, Vector<Type>)>,
    pub monomorphised_funcs: HashSet<(Name, Vector<Type>)>,
    pub stmts: Vector<Stmt>,
    pub diagnostics: Diagnostics,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            stack: Stack::new(()),
            monomorphised_types: HashSet::new(),
            monomorphised_funcs: HashSet::new(),
            stmts: Vector::new(),
            diagnostics: Diagnostics::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TypeDecl {
    Enum(Info, Meta, Vector<Name>, Vector<(Name, Type)>),
    Bit(Info, Meta, Vector<Name>),
    Generic(Type),
}

#[derive(Debug, Clone)]
pub(crate) enum ExprDecl {
    Def(Info, Meta, Vector<Name>, Vector<Pattern>, Type, Block),
    Bif(Info, Meta, Vector<Name>, Vector<Type>, Type),
    Var(Type),
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }
}
