//! HIR -> Rust:
//! * Replace rows with structs
#![allow(unused)]
pub mod context;

use std::io::Read;

use builtins::time_source::TimeSource;
use context::FuncDecl;
use context::TypeDecl;
use diagnostics::Error;
use hir::*;
use im_rc::ordmap;
use im_rc::vector;
use im_rc::OrdMap;
use im_rc::Vector;
use info::Info;
use utils::OptionUtils;
use utils::VectorUtils;
use value::dynamic::Dataflow;
use value::dynamic::Sink;
use value::dynamic::Stream;
use value::dynamic::StreamKind::DApply;
use value::dynamic::StreamKind::DFilter;
use value::dynamic::StreamKind::DFlatMap;
use value::dynamic::StreamKind::DFlatten;
use value::dynamic::StreamKind::DKeyby;
use value::dynamic::StreamKind::DMap;
use value::dynamic::StreamKind::DMerge;
use value::dynamic::StreamKind::DScan;
use value::dynamic::StreamKind::DSource;
use value::dynamic::StreamKind::DUnkey;
use value::dynamic::StreamKind::DWindow;

use crate::context::Context;

pub fn process(ctx: &mut Context, d: Dataflow, ss: Vector<Stmt>) -> Vector<Stmt> {
    ss.into_iter().for_each(|s| visit_top_stmt(ctx, s));
    for stream in d.streams {
        visit_stream(ctx, stream);
    }
    for x in d.sinks {
        visit_sink(ctx, x);
    }
    std::mem::take(&mut ctx.stmts)
}

fn visit_sink(ctx: &mut Context, sink: Sink) {
    let (x, w, e) = sink.0.as_ref();
}

fn visit_stream(ctx: &mut Context, stream: Stream) {
    match stream.kind.as_ref().clone() {
        DSource(_, _, a) => match a {
            TimeSource::Ingestion { watermark_interval } => {}
            TimeSource::Event {
                extractor,
                watermark_interval,
                slack,
            } => {
                reach_func(ctx, extractor.x, extractor.ts);
            }
        },
        DMap(_, f) => {
            reach_func(ctx, f.x, f.ts);
        }
        DFilter(_, f) => {
            reach_func(ctx, f.x, f.ts);
        }
        DFlatten(_) => {}
        DFlatMap(_, f) => {
            reach_func(ctx, f.x, f.ts);
        }
        DScan(_, f) => {
            reach_func(ctx, f.x, f.ts);
        }
        DKeyby(_, f) => {
            reach_func(ctx, f.x, f.ts);
        }
        DUnkey(_) => {}
        DApply(_, f) => {
            reach_func(ctx, f.x, f.ts);
        }
        DWindow(a0, a1, a2) => {
            todo!()
        }
        DMerge(_) => {}
    }
}

fn visit_top_stmt(ctx: &mut Context, s: Stmt) {
    let info = s.info;
    match s.kind {
        SMonoDef(m, x, ts, ps, t, b) => {
            ctx.funcs.insert((x, ts), FuncDecl::Def(info, m, ps, t, b));
        }
        SMonoBif(m, x, ts0, ts1, t) => {
            ctx.funcs.insert((x, ts0), FuncDecl::Bif(info, m, ts1, t));
        }
        SMonoEnum(m, x, ts, xts) => {
            ctx.types.insert((x, ts), TypeDecl::Enum(info, m, xts));
        }
        SMonoBit(m, x, ts) => {
            ctx.types.insert((x, ts), TypeDecl::Bit(info, m));
        }
        SVar(..) => {
            ctx.diagnostics.push_error(Error::UncompileableCode {
                info,
                msg: "Top level statements are not allowed",
            });
        }
        SRecDef(..) => todo!(),
        SNoop => {}
        SDef(_, _, _, _, _, _) => todo!(),
        SBif(_, _, _, _, _) => todo!(),
        SEnum(_, _, _, _) => todo!(),
        SBit(_, _, _) => todo!(),
    }
}

fn visit_type(ctx: &mut Context, t: Type) {
    match t.kind() {
        TFun(ts, t) => {
            ts.into_iter().for_each(|t| visit_type(ctx, t));
            visit_type(ctx, t);
        }
        TRecord(t) => {
            visit_type(ctx, t);
        }
        TArray(t, _) => {
            visit_type(ctx, t);
        }
        TNominal(x, ts) => match ctx.types.get(&(x.clone(), ts.clone())).cloned() {
            Some(TypeDecl::Bit(info, m)) => {
                if !ctx.reachable.contains(&(x.clone(), ts.clone())) {
                    ctx.reachable.insert((x.clone(), ts.clone()));
                    ctx.stmts.push_back(SMonoBit(m, x, ts).with(info));
                }
            }
            Some(TypeDecl::Enum(info, m, xts)) => {
                if !ctx.reachable.contains(&(x.clone(), ts.clone())) {
                    ctx.reachable.insert((x.clone(), ts.clone()));
                    xts.clone()
                        .into_iter()
                        .for_each(|(x, t)| visit_type(ctx, t));
                    ctx.stmts.push_back(SMonoEnum(m, x, ts, xts).with(info));
                }
            }
            None => unreachable!("Type should be declared by now {:?}", x),
        },
        TRowEmpty => {}
        TRowExtend((x, t), r) => {
            visit_type(ctx, t);
            visit_type(ctx, r);
        }
        TAlias(_, _, t) => visit_type(ctx, t),
        TVar(..) | TGeneric(..) | TRecordConcat(..) | TTuple(..) | TArrayConcat(..) => {
            todo!()
        }
        TUnit => {}
        TNever => {}
        TError => {}
    }
}

fn visit_pattern(ctx: &mut Context, p: Pattern) {
    let t = visit_type(ctx, p.t);
}

fn visit_expr_stmt(ctx: &mut Context, s: Stmt) {
    let e = match s.kind {
        SVar(p, e) => {
            visit_pattern(ctx, p);
            e
        }
        x => unreachable!("{:?}", x),
    };
    match e.kind() {
        EConst(_) => {}
        EFunCall(..) => {}
        EFunCallDirect(x, ts, es) => reach_func(ctx, x, ts),
        EFunReturn(_) => {}
        EBreak(_) => {}
        EIfElse(e, b0, b1) => {
            visit_block(ctx, b0);
            visit_block(ctx, b1);
        }
        ERecord(_) => {}
        ERecordConcat(_, _) => {}
        ERecordAccess(_, _) => {}
        EDef(x, ts) => reach_func(ctx, x, ts),
        EVariant(_, _, _, _) => {}
        EVariantAccess(_, _, _, _) => {}
        EVariantCheck(_, _, _, _) => {}
        ESsaLoop(pes, b) => {
            pes.into_iter().for_each(|(p, _)| visit_pattern(ctx, p));
            visit_block(ctx, b);
        }
        ESsaContinue(_) => {}
        EArray(_) => {}
        EArrayConcat(_, _) => todo!(),
        ELoop(_)
        | EContinue
        | EArrayAccess(_, _)
        | EFun(_, _, _)
        | EMatch(_, _)
        | EMut(_, _)
        | EVar(_)
        | EDo(_)
        | ENoop(_)
        | ETuple(_)
        | ETupleAccess(_, _)
        | EFor(_, _, _)
        | EWhile(_, _) => {
            unreachable!("{:?}", e)
        }
        EError => unreachable!(),
    }
}

fn visit_block(ctx: &mut Context, b: Block) {
    b.ss.into_iter().for_each(|s| visit_expr_stmt(ctx, s));
}

fn reach_func(ctx: &mut Context, x: Name, ts: Vector<Type>) {
    match ctx.funcs.get(&(x.clone(), ts.clone())).cloned() {
        Some(FuncDecl::Def(info, m, ps, t, b)) => {
            if !ctx.reachable.contains(&(x.clone(), ts.clone())) {
                ctx.reachable.insert((x.clone(), ts.clone()));
                ps.clone().into_iter().for_each(|p| visit_pattern(ctx, p));
                visit_type(ctx, t.clone());
                visit_block(ctx, b.clone());
                ctx.stmts
                    .push_back(SDef(m, x, vector![], ps, t, b).with(info));
            }
        }
        Some(FuncDecl::Bif(info, m, ts, t)) => {
            if !ctx.reachable.contains(&(x.clone(), ts.clone())) {
                ctx.reachable.insert((x.clone(), ts.clone()));
                ts.clone().into_iter().for_each(|t| visit_type(ctx, t));
                visit_type(ctx, t.clone());
                ctx.stmts.push_back(SBif(m, x, vector![], ts, t).with(info));
            }
        }
        y => unreachable!("{:?} {:?}", x, y),
    }
}

fn reach_type(ctx: &mut Context, x: Name, ts: Vector<Type>) {
    match ctx.types.get(&(x.clone(), ts.clone())).cloned() {
        Some(TypeDecl::Bit(info, m)) => {
            if !ctx.reachable.contains(&(x.clone(), ts.clone())) {
                ctx.reachable.insert((x.clone(), ts.clone()));
                ctx.stmts.push_back(SBit(m, x, vector![]).with(info));
            }
        }
        Some(TypeDecl::Enum(info, m, xts)) => {
            if !ctx.reachable.contains(&(x.clone(), ts.clone())) {
                ctx.reachable.insert((x.clone(), ts.clone()));
                xts.clone()
                    .into_iter()
                    .for_each(|(_, t)| visit_type(ctx, t));
                ctx.stmts.push_back(SEnum(m, x, vector![], xts).with(info));
            }
        }
        y => unreachable!("{:?} {:?}", x, y),
    }
}
