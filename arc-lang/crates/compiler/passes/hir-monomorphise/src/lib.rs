#![allow(unused)]
pub mod context;

use crate::context::Context;
use context::ExprDecl;
use context::TypeDecl;
use diagnostics::Error;
use hir::*;
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
        visit_top_stmt(ctx, s);
    }
    std::mem::take(&mut ctx.stmts)
}

fn visit_top_stmt(ctx: &mut Context, s: Stmt) {
    let info = s.info;
    match s.kind {
        SDef(m, x, gs, ps, t, b) => {
            ctx.stack
                .bind_expr_decl(x, ExprDecl::Def(info, m, gs, ps, t, b));
        }
        SRecDef(_, _) => todo!(),
        SBif(m, x, gs, ts, t) => {
            ctx.stack
                .bind_expr_decl(x, ExprDecl::Bif(info, m, gs, ts, t));
        }
        SEnum(m, x, gs, xts) => {
            ctx.stack
                .bind_type_decl(x, TypeDecl::Enum(info, m, gs, xts));
        }
        SBit(m, x, gs) => {
            ctx.stack.bind_type_decl(x, TypeDecl::Bit(info, m, gs));
        }
        SVar(p, e) => {
            let e = visit_expr(ctx, e);
            let p = visit_val_pattern(ctx, p);
            ctx.stmts.push_back(SVar(p, e).with(s.info));
        }
        SNoop => {}
        SMonoDef(_, _, _, _, _, _) => unreachable!(),
        SMonoBif(_, _, _, _, _) => unreachable!(),
        SMonoEnum(_, _, _, _) => unreachable!(),
        SMonoBit(_, _, _) => unreachable!(),
    }
}

fn visit_type(ctx: &mut Context, t: Type, info: Info) -> Type {
    match t.kind() {
        TFun(ts, t) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            let t = visit_type(ctx, t, info);
            TFun(ts, t).into()
        }
        TRecord(t) => {
            let t = visit_type(ctx, t, info);
            TRecord(t).into()
        }
        TRecordConcat(t0, t1) => {
            let t0 = visit_type(ctx, t0, info);
            let t1 = visit_type(ctx, t1, info);
            TRecordConcat(t0, t1).into()
        }
        TNominal(x, ts) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            monomorphise_type(ctx, x.clone(), ts.clone());
            TNominal(x, ts).into()
        }
        TGeneric(x) => match ctx.stack.find_type_decl(&x) {
            Some(TypeDecl::Generic(t)) => t,
            x => unreachable!("{:?}", x),
        },
        TRowEmpty => TRowEmpty.into(),
        TRowExtend((x, t), r) => {
            let t = visit_type(ctx, t, info);
            let r = visit_type(ctx, r, info);
            TRowExtend((x, t), r).into()
        }
        TTuple(ts, c) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            TTuple(ts, c).into()
        }
        TAlias(_, _, t) => visit_type(ctx, t, info),
        TArray(t, c) => {
            let t = visit_type(ctx, t, info);
            TArray(t, c).into()
        }
        TArrayConcat(t0, t1) => {
            let t0 = visit_type(ctx, t0, info);
            let t1 = visit_type(ctx, t1, info);
            TArrayConcat(t0, t1).into()
        }
        TUnit => TUnit.into(),
        TNever => TNever.into(),
        TVar(x) => TVar(x).into(),
        TError => TError.into(),
    }
}

fn visit_val_pattern(ctx: &mut Context, p: Pattern) -> Pattern {
    let info = p.info;
    let t = visit_type(ctx, p.t.clone(), info);
    match p.kind() {
        PIgnore => PIgnore.with(t, info),
        PVar(x) => {
            ctx.stack
                .bind_expr_decl(x.clone(), ExprDecl::Var(t.clone()));
            PVar(x).with(t, info)
        }
        _ => unreachable!(),
    }
}

fn visit_val_expr(ctx: &mut Context, e: Expr) -> Expr {
    let info = e.info;
    match e.kind.as_ref().clone() {
        EVar(x) => match ctx.stack.find_expr_decl(&x).unwrap() {
            ExprDecl::Var(t) => EVar(x).with(t, info),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn visit_expr(ctx: &mut Context, e: Expr) -> Expr {
    let info = e.info;
    let t = visit_type(ctx, e.t.clone(), info);
    match e.kind() {
        EConst(c) => EConst(c).with(t, info),
        EFunCallDirect(x, ts, es) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            let es = es.map(|e| visit_val_expr(ctx, e));
            monomorphise_func(ctx, x.clone(), ts.clone());
            EFunCallDirect(x, ts, es).with(t, info)
        }
        EDef(x, ts) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            monomorphise_func(ctx, x.clone(), ts.clone());
            EDef(x, ts).with(t, info)
        }
        EFunCall(e, es) => {
            let e = visit_val_expr(ctx, e);
            let es = es.map(|e| visit_val_expr(ctx, e));
            EFunCall(e, es).with(t, info)
        }
        EFunReturn(e) => {
            let e = visit_val_expr(ctx, e);
            EFunReturn(e).with(t, info)
        }
        EIfElse(e, b0, b1) => {
            let e = visit_val_expr(ctx, e);
            let b0 = visit_block(ctx, b0);
            let b1 = visit_block(ctx, b1);
            EIfElse(e, b0, b1).with(t, info)
        }
        ERecord(xes) => {
            let xes = xes.map(|(x, e)| (x, visit_val_expr(ctx, e)));
            ERecord(xes).with(t, info)
        }
        ERecordConcat(e0, e1) => {
            let e0 = visit_val_expr(ctx, e0);
            let e1 = visit_val_expr(ctx, e1);
            ERecordConcat(e0, e1).with(t, info)
        }
        ERecordAccess(e, x) => {
            let e = visit_val_expr(ctx, e);
            ERecordAccess(e, x).with(t, info)
        }
        EVariant(x0, ts, x1, e) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            monomorphise_type(ctx, x0.clone(), ts.clone());
            EVariant(x0, ts, x1, e).with(t, info)
        }
        EVariantAccess(x0, ts, x1, e) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            monomorphise_type(ctx, x0.clone(), ts.clone());
            EVariantAccess(x0, ts, x1, e).with(t, info)
        }
        EVariantCheck(x0, ts, x1, v) => {
            let ts = ts.map(|t| visit_type(ctx, t, info));
            monomorphise_type(ctx, x0.clone(), ts.clone());
            EVariantCheck(x0, ts, x1, v).with(t, info)
        }
        EArray(es) => {
            let es = es.mapm(ctx, visit_val_expr);
            EArray(es).with(t, info)
        }
        EArrayConcat(e0, e1) => {
            let e0 = visit_val_expr(ctx, e0);
            let e1 = visit_val_expr(ctx, e1);
            EArrayConcat(e0, e1).with(t, info)
        }
        EArrayAccess(e0, e1) => {
            let e0 = visit_val_expr(ctx, e0);
            let e1 = visit_val_expr(ctx, e1);
            EArrayAccess(e0, e1).with(t, info)
        }
        EMut(e0, e1) => {
            let e0 = visit_val_expr(ctx, e0);
            let e1 = visit_val_expr(ctx, e1);
            EMut(e0, e1).with(t, info)
        }
        EVar(x) => EVar(x).with(t, info),
        ETuple(es) => {
            let es = es.mapm(ctx, visit_val_expr);
            ETuple(es).with(t, info)
        }
        ETupleAccess(e, n) => {
            let e = visit_val_expr(ctx, e);
            ETupleAccess(e, n).with(t, info)
        }
        ENoop(e) => visit_val_expr(ctx, e),
        ESsaLoop(pes, b) => {
            let pes = pes.map(|(p, e)| {
                let p = visit_val_pattern(ctx, p);
                let e = visit_val_expr(ctx, e);
                (p, e)
            });
            let b = visit_block(ctx, b);
            ESsaLoop(pes, b).with(t, info)
        }
        EBreak(e) => {
            let e = visit_val_expr(ctx, e);
            EBreak(e).with(t, info)
        }
        ESsaContinue(pes) => {
            let pes = pes.map(|(p, e)| {
                let p = visit_val_pattern(ctx, p);
                let e = visit_val_expr(ctx, e);
                (p, e)
            });
            ESsaContinue(pes).with(t, info)
        }
        EError => EError.with(t, info),
        EDo(_) => unreachable!(),
        EFun(_, _, _) => unreachable!(),
        EMatch(_, _) => unreachable!(),
        EFor(_, _, _) => unreachable!(),
        EWhile(_, _) => unreachable!(),
        ELoop(_) => unreachable!(),
        EContinue => unreachable!(),
    }
}

fn visit_block(ctx: &mut Context, b: Block) -> Block {
    let ss = b.ss.mapm(ctx, visit_expr_stmt);
    let e = visit_expr(ctx, b.e);
    Block::new(ss, e, b.info)
}

fn visit_expr_stmt(ctx: &mut Context, s: Stmt) -> Stmt {
    if let SVar(p, e) = s.kind {
        let e = visit_expr(ctx, e);
        let p = visit_val_pattern(ctx, p);
        SVar(p, e).with(s.info)
    } else {
        unreachable!()
    }
}

fn monomorphise_func(ctx: &mut Context, x: Name, ts: Vector<Type>) {
    if ctx
        .monomorphised_funcs
        .insert((x.clone(), ts.clone()))
        .is_some()
    {
        return;
    }
    match ctx
        .stack
        .find_expr_decl(&x)
        .expect(&format!("{} not found", x))
    {
        ExprDecl::Def(info, m, gs, ps, t, b) => {
            ctx.stack.push_scope(());
            for (g, t) in gs.into_iter().zip(ts.clone().into_iter()) {
                ctx.stack.bind_type_decl(g, TypeDecl::Generic(t));
            }
            let ps = ps.mapm(ctx, visit_val_pattern);
            let t = visit_type(ctx, t, info);
            let b = visit_block(ctx, b);
            ctx.stack.pop_scope();
            ctx.stmts.push_back(SMonoDef(m, x, ts, ps, t, b).with(info));
        }
        ExprDecl::Bif(info, m, gs, ts1, t) => {
            ctx.stack.push_scope(());
            for (g, t) in gs.into_iter().zip(ts.into_iter()) {
                ctx.stack.bind_type_decl(g, TypeDecl::Generic(t));
            }
            let ts1 = ts1.map(|t| visit_type(ctx, t, info));
            let t = visit_type(ctx, t, info);
            ctx.stack.pop_scope();
            ctx.stmts
                .push_back(SMonoBif(m, x, vector![], ts1, t).with(info));
        }
        x => unreachable!("{:?}", x),
    }
}

fn monomorphise_type(ctx: &mut Context, x: Name, ts: Vector<Type>) {
    if ctx
        .monomorphised_types
        .insert((x.clone(), ts.clone()))
        .is_some()
    {
        return;
    }
    match ctx
        .stack
        .find_type_decl(&x)
        .expect(&format!("{} not found", x))
    {
        TypeDecl::Enum(info, m, gs, xts) => {
            ctx.stack.push_scope(());
            for (g, t) in gs.into_iter().zip(ts.clone().into_iter()) {
                ctx.stack.bind_type_decl(g, TypeDecl::Generic(t));
            }
            let xts = xts.map(|(x, t)| (x, visit_type(ctx, t, info)));
            ctx.stack.pop_scope();
            ctx.stmts.push_back(SMonoEnum(m, x, ts, xts).with(info));
        }
        TypeDecl::Bit(info, m, _) => {
            ctx.stmts.push_back(SMonoBit(m, x, ts).with(info));
        }
        x => unreachable!("{:?}", x),
    }
}
