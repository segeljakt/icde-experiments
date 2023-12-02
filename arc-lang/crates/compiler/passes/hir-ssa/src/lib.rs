#![allow(unused)]
pub mod context;

use crate::context::Context;
use context::Scope;
use context::ScopeKind;
use diagnostics::Error;
use hir::*;
use im_rc::HashMap;
use im_rc::hashmap::Entry;
use im_rc::ordmap;
use im_rc::vector;
use im_rc::OrdMap;
use im_rc::Vector;
use info::Info;
use utils::OrdMapUtils;
use utils::VectorUtils;

pub fn process(ctx: &mut Context, ss: Vector<Stmt>) -> Vector<Stmt> {
    for s in ss {
        visit_stmt(ctx, s);
    }
    ctx.take_stmts()
}

fn visit_stmt(ctx: &mut Context, s: Stmt) {
    let info = s.info;
    match s.kind {
        SDef(m, x, gs, ps, t, b) => {
            let ps = ps.mapm(ctx, visit_pattern);
            let b = visit_def_block(ctx, b);
            ctx.add_stmt(SDef(m, x, gs, ps, t, b).with(info));
        }
        SRecDef(_, _) => todo!(),
        SBif(m, x, gs, ts, t) => {
            ctx.add_stmt(SBif(m, x, gs, ts, t).with(info));
        }
        SEnum(m, x, gs, xts) => {
            ctx.add_stmt(SEnum(m, x, gs, xts).with(info));
        }
        SBit(m, x, gs) => {
            ctx.add_stmt(SBit(m, x, gs).with(info));
        }
        SVar(p, e) => {
            let e = visit_expr_stmt(ctx, e);
            let p = visit_pattern(ctx, p);
            ctx.add_stmt(SVar(p, e).with(s.info));
        }
        SNoop => {}
        SMonoDef(_, _, _, _, _, _) => unreachable!(),
        SMonoBif(_, _, _, _, _) => unreachable!(),
        SMonoEnum(_, _, _, _) => unreachable!(),
        SMonoBit(_, _, _) => unreachable!(),
    }
}

fn visit_pattern(ctx: &mut Context, p: Pattern) -> Pattern {
    todo!()
    // if let PVar(x) = p.kind() {
    //     let n = 0;
    //     ctx.stack.bind_expr_decl(x.clone(), (n, p.t.clone()));
    //     PVar(format!("{x}{n}")).with(p.t, p.info)
    // } else {
    //     p
    // }
}

fn visit_rval_expr(ctx: &mut Context, e: Expr) -> Expr {
    todo!()
    // let info = e.info;
    // match e.kind.as_ref().clone() {
    //     EVar(x) => e,
    //     EVar(x) => {
    //         let (n, _) = ctx.stack.find_expr_decl(&x).unwrap();
    //         EVar(format!("{x}{n}")).with(e.t, info)
    //     }
    //     _ => unreachable!(),
    // }
}

fn visit_lval_expr(ctx: &mut Context, e0: Expr, e1: Expr) -> Expr {
    todo!()
    // let info = e0.info;
    // match e0.kind() {
    //     EVar(x0) => e0,
    //     EVar(x0) => {
    //         let (n, _) = ctx.stack.find_expr_decl_mut(&x0).unwrap();
    //         *n += 1;
    //         let x = format!("{x0}{n}");
    //         ctx.add_stmt(SVar(PVar(x.clone()).with(e0.t.clone(), info), e1).with(info));
    //         EVar(x).with(e0.t, info)
    //     }
    //     _ => unreachable!(),
    // }
}

fn visit_expr_stmt(ctx: &mut Context, e: Expr) -> Expr {
    let info = e.info;
    let t = e.t.clone();
    match e.kind() {
        EConst(c) => EConst(c).with(t, info),
        EFunCallDirect(x, ts, es) => {
            let es = es.into_iter().map(|e| visit_rval_expr(ctx, e)).collect();
            EFunCallDirect(x, ts, es).with(t, info)
        }
        EDef(x, ts) => EDef(x, ts).with(t, info),
        EFunCall(e, es) => {
            let e = visit_rval_expr(ctx, e);
            let es = es.into_iter().map(|e| visit_rval_expr(ctx, e)).collect();
            EFunCall(e, es).with(t, info)
        }
        EFunReturn(e) => {
            let e = visit_rval_expr(ctx, e);
            EFunReturn(e).with(t, info)
        }
        EIfElse(e, b0, b1) => {
            let e = visit_rval_expr(ctx, e);
            let b0 = visit_def_block(ctx, b0);
            let b1 = visit_def_block(ctx, b1);
            EIfElse(e, b0, b1).with(t, info)
        }
        ELoop(b) => {
            todo!()
            // let (ms0, ms1) = ctx.live_vars();
            // let pes = ms0.zip(&ms1).map(|((x0, (n0, t0)), (x1, (n1, t1)))| {
            //     let p = PVar(format!("{x1}{n1}")).with(t1.clone(), info);
            //     let e = EVar(format!("{x0}{n0}")).with(t0, info);
            //     (p, e)
            // });
            // ctx.stack
            //     .push_scope(Scope::new(ScopeKind::Loop(ms1.clone())));
            // b.ss.into_iter().for_each(|s| visit_stmt(ctx, s));
            // let e = visit_expr_stmt(ctx, b.e);
            // assert!(matches!(e.kind.as_ref(), EVar(_)));
            // let pes1 = ms1.clone().map(|(x, (n0, t0))| {
            //     let (n1, t1) = ctx.stack.find_expr_decl(&x).unwrap();
            //     let p = PVar(format!("{x}{n0}")).with(t0, info);
            //     let e = EVar(format!("{x}{n1}")).with(t1, info);
            //     (p, e)
            // });
            // let ss = ctx.stack.pop_scope().stmts;
            // let b = Block::new(ss, ESsaContinue(pes1).with(e.t.clone(), info), b.info);
            // ctx.reset_mutables(ms1);
            // ESsaLoop(pes, b).with(t, info)
        }
        EBreak(e) => {
            todo!()
            // let e = visit_rval_expr(ctx, e);
            // let vs = ctx.loop_vars();
            // let mut es = vs.clone().map(|(x, (n0, t0))| {
            //     let (n1, t1) = ctx.stack.find_expr_decl(&x).unwrap();
            //     EVar(format!("{x}{n1}")).with(t1, info)
            // });
            // es.push_front(e);
            // let ts = vs.clone().map(|(_, (_, t))| t);
            // let t1 = TTuple(ts, true).into();
            // let e1 = ETuple(es).with(t1, info);
            // EBreak(e1).with(t, info)
        }
        EContinue => {
            todo!()
            // let e = visit_rval_expr(ctx, e);
            // let ms = ctx.loop_vars();
            // let pes = ms.map(|(x, (n0, t0))| {
            //     let (n1, t1) = ctx.stack.find_expr_decl(&x).unwrap();
            //     let p = PVar(format!("{x}{n0}")).with(t0, info);
            //     let e = EVar(format!("{x}{n1}")).with(t1, info);
            //     (p, e)
            // });
            // ESsaContinue(pes).with(t, info)
        }
        ERecord(xes) => {
            let xes = xes.map(|(x, e)| (x, visit_rval_expr(ctx, e)));
            ERecord(xes).with(t, info)
        }
        ERecordConcat(e0, e1) => {
            let e0 = visit_rval_expr(ctx, e0);
            let e1 = visit_rval_expr(ctx, e1);
            ERecordConcat(e0, e1).with(t, info)
        }
        ERecordAccess(e, x) => {
            let e = visit_rval_expr(ctx, e);
            ERecordAccess(e, x).with(t, info)
        }
        EVariant(x0, ts, x1, e) => {
            let e = visit_rval_expr(ctx, e);
            EVariant(x0, ts, x1, e).with(t, info)
        }
        EVariantAccess(x0, ts, x1, e) => {
            let e = visit_rval_expr(ctx, e);
            EVariantAccess(x0, ts, x1, e).with(t, info)
        }
        EVariantCheck(x0, ts, x1, v) => {
            let v = visit_rval_expr(ctx, v);
            EVariantCheck(x0, ts, x1, v).with(t, info)
        }
        EArray(es) => {
            let es = es.mapm(ctx, visit_rval_expr);
            EArray(es).with(t, info)
        }
        EArrayConcat(e0, e1) => {
            let e0 = visit_rval_expr(ctx, e0);
            let e1 = visit_rval_expr(ctx, e1);
            EArrayConcat(e0, e1).with(t, info)
        }
        EArrayAccess(e0, e1) => {
            let e0 = visit_rval_expr(ctx, e0);
            let e1 = visit_rval_expr(ctx, e1);
            EArrayAccess(e0, e1).with(t, info)
        }
        EMut(e0, e1) => {
            let e1 = visit_rval_expr(ctx, e1);
            let e0 = visit_lval_expr(ctx, e0, e1);
            EConst(CUnit).with(t, info)
        }
        EVar(x) => EVar(x).with(t, info),
        EVar(x) => {
            todo!()
            // let (n, _) = ctx.stack.find_expr_decl(&x).unwrap();
            // EVar(format!("{x}{n}")).with(t, info)
        }
        ETuple(es) => {
            let es = es.mapm(ctx, visit_rval_expr);
            ETuple(es).with(t, info)
        }
        ETupleAccess(e, n) => {
            let e = visit_rval_expr(ctx, e);
            ETupleAccess(e, n).with(t, info)
        }
        ENoop(e) => visit_rval_expr(ctx, e),
        EError => EError.with(t, info),
        EDo(_) => unreachable!(),
        EFun(_, _, _) => unreachable!(),
        EMatch(_, _) => unreachable!(),
        EFor(_, _, _) => unreachable!(),
        EWhile(_, _) => unreachable!(),
        ESsaLoop(_, _) => unreachable!(),
        ESsaContinue(_) => unreachable!(),
    }
}

// Declared in inner scope - Declared in current scope
fn visit_def_block(ctx: &mut Context, b: Block) -> Block {
    ctx.stack.push_scope(Scope::new(ScopeKind::Def, HashMap::new()));
    b.ss.into_iter().for_each(|s| visit_stmt(ctx, s));
    let e = visit_expr_stmt(ctx, b.e);
    let ss = ctx.stack.pop_scope().stmts;
    Block::new(ss, e, b.info)
}

// Returns all the variables that are mutated in the expression
fn mutated(b: &Block, xs0: Vector<Name>) -> Vector<Name> {
    fn block(b: &Block, xs0: &Vector<Name>, xs1: &mut Vector<Name>) {
        b.ss.iter().for_each(|s| stmt(s, xs0, xs1));
    }
    fn stmt(s: &Stmt, xs0: &Vector<Name>, xs1: &mut Vector<Name>) {
        if let SVar(_, e) = &s.kind {
            expr(e, xs0, xs1)
        }
    }
    fn expr(e: &Expr, xs0: &Vector<Name>, xs1: &mut Vector<Name>) {
        match e.kind.as_ref() {
            ELoop(b) => for s in &b.ss {},
            EIfElse(_, b0, b1) => {
                block(b0, xs0, xs1);
                block(b1, xs0, xs1);
            }
            EMut(e, _) => match e.kind.as_ref() {
                EVar(x) => {
                    if xs0.contains(x) && !xs1.contains(x) {
                        xs1.push_back(x.clone());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    let mut xs1 = Vector::new();
    block(b, &xs0, &mut xs1);
    xs1
}
