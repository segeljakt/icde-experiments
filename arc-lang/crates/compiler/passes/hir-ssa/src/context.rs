use std::collections::HashSet;

use diagnostics::Diagnostics;
use hir::*;
use im_rc::HashMap;
use im_rc::OrdMap;
use im_rc::Vector;
use info::Info;
use stack::Stack;

#[derive(Debug, Clone)]
pub struct Context {
    pub(crate) stack: Stack<Scope, (), ()>,
    pub diagnostics: Diagnostics,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub stmts: Vector<Stmt>,
    pub versions: HashMap<Name, usize>,
    pub kind: ScopeKind,
}

#[derive(Debug, Clone)]
pub enum ScopeKind {
    Loop(Vector<(Name, (usize, Type))>),
    Def,
    Block,
}

impl Scope {
    pub fn new(kind: ScopeKind, versions: HashMap<Name, usize>) -> Scope {
        Scope {
            stmts: Vector::new(),
            versions,
            kind,
        }
    }
}

impl Default for Context {
    fn default() -> Context {
        Context {
            stack: Stack::new(Scope::new(ScopeKind::Block, HashMap::new())),
            diagnostics: Diagnostics::default(),
        }
    }
}

impl Context {
    pub fn new() -> Context {
        Context::default()
    }

    pub(crate) fn add_stmt(&mut self, s: Stmt) {
        self.stack.current().stmts.push_back(s);
    }

    pub(crate) fn add_stmts(&mut self, ss: Vector<Stmt>) {
        self.stack.current().stmts.extend(ss);
    }

    pub(crate) fn take_stmts(&mut self) -> Vector<Stmt> {
        std::mem::take(&mut self.stack.current().stmts)
    }

    // pub(crate) fn snapshot(&self) -> Scope {
    //     self.stack.current().clone()
    // }

    pub(crate) fn enter_scope(&mut self, kind: ScopeKind) {
        let versions = self.stack.current().versions.clone();
        self.stack.push_scope(Scope::new(kind, versions));
    }

    pub(crate) fn exit_scope(&mut self) -> Scope {
        self.stack.pop_scope()
    }

    pub(crate) fn increment(&mut self, x: &Name) -> usize {
        let n = self.stack.current().versions.get_mut(x).unwrap();
        *n += 1;
        *n
    }

    pub(crate) fn bind(&mut self, x: Name) {
        self.stack.current().versions.insert(x, 0);
    }

    pub(crate) fn changed(
        &self,
        a: &Stack<Scope, (usize, Type), ()>,
        b: &Stack<Scope, (usize, Type), ()>,
    ) -> Stack<Scope, (usize, Type), ()> {
        todo!()
    }

    // pub(crate) fn mutables(&self) -> Vector<(Name, (usize, Type))> {
    //     let mut mutables = Vector::new();
    //     for scope in self.stack.iter() {
    //         mutables.extend(scope.expr_namespace.iter().cloned());
    //         match scope.kind.kind {
    //             ScopeKind::Def => break,
    //             ScopeKind::Loop(_) => continue,
    //             ScopeKind::Block => continue,
    //         }
    //     }
    //     mutables
    // }
    //
    // pub(crate) fn live_vars(
    //     &mut self,
    // ) -> (Vector<(Name, (usize, Type))>, Vector<(Name, (usize, Type))>) {
    //     let mut ms0 = Vector::new();
    //     let mut ms1 = Vector::new();
    //     for scope in self.stack.iter_mut() {
    //         for (x, (n, t)) in scope.expr_namespace.iter_mut() {
    //             ms0.push_back((x.clone(), (*n, t.clone())));
    //             *n += 1;
    //             ms1.push_back((x.clone(), (*n, t.clone())));
    //         }
    //         match scope.kind.kind {
    //             ScopeKind::Def => break,
    //             ScopeKind::Loop(_) => continue,
    //             ScopeKind::Block => continue,
    //         }
    //     }
    //     (ms0, ms1)
    // }
    //
    // pub(crate) fn reset_mutables(&mut self, mutables: Vector<(Name, (usize, Type))>) {
    //     for (x, (n, t)) in mutables {
    //         self.stack.bind_expr_decl(x, (n, t));
    //     }
    // }
    //
    // pub(crate) fn loop_vars(&self) -> Vector<(Name, (usize, Type))> {
    //     for scope in self.stack.iter() {
    //         match &scope.kind.kind {
    //             ScopeKind::Def => unreachable!("ICE: Def scope in loop_mutables"),
    //             ScopeKind::Loop(xs) => return xs.clone(),
    //             ScopeKind::Block => continue,
    //         }
    //     }
    //     unreachable!("ICE: No loop scope in loop_mutables");
    // }
}
