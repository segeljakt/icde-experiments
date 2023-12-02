#![allow(unused)]

pub mod context;
pub mod definitions;

use context::FuncDecl;
use context::TypeDecl;
use diagnostics::Error;
use hir::*;
use im_rc::vector;
use im_rc::OrdMap;
use im_rc::Vector;
use utils::AssocVectorUtils;
use utils::VectorUtils;
use value::dynamic::Array;
use value::dynamic::Function;
use value::dynamic::Record;
use value::dynamic::Tuple;
use value::dynamic::Variant;
use value::Value;
use value::ValueKind::VTuple;
use value::ValueKind::VUnit;

use crate::context::Context;

pub fn process(ctx: &mut Context, ss: Vector<Stmt>) {
    for s in ss {
        eval_top_stmt(ctx, s.clone());
        ctx.ss.push_back(s);
    }
}

fn eval_top_stmt(ctx: &mut Context, s: Stmt) -> Result<Option<(Value, Type)>, Error> {
    match s.kind {
        SMonoDef(m, x, ts, ps, t, b) => {
            ctx.funcs.insert((x, ts), FuncDecl::Def(m, ps, t, b));
        }
        SMonoBif(m, x, ts0, ts1, t) => {
            ctx.funcs.insert((x, ts0), FuncDecl::Bif(m, ts1, t));
        }
        SMonoEnum(m, x, ts, xts) => {
            ctx.types.insert((x, ts), TypeDecl::Enum(xts));
        }
        SMonoBit(m, x, ts) => {
            ctx.types.insert((x, ts), TypeDecl::Bit);
        }
        SVar(p, e) => {
            let t = e.t.clone();
            match eval_expr(ctx, e) {
                CValue(v) => {
                    bind(ctx, p, v.clone());
                    return Ok(Some((v, t)));
                }
                CException(e) => return Err(e),
                _ => unreachable!(),
            }
        }
        SRecDef(..) => todo!(),
        SNoop => unreachable!(),
        SDef(_, _, _, _, _, _) => todo!(),
        SBif(_, _, _, _, _) => todo!(),
        SEnum(_, _, _, _) => todo!(),
        SBit(_, _, _) => todo!(),
    }
    Ok(None)
}

fn eval_expr_stmt(ctx: &mut Context, s: Stmt) -> Control {
    if let SVar(p, e) = s.kind {
        let c = eval_expr(ctx, e);
        if let CValue(v) = c {
            bind(ctx, p, v);
            CValue(().into())
        } else {
            c
        }
    } else {
        unreachable!()
    }
}

fn bind(ctx: &mut Context, p: Pattern, v: Value) {
    match p.kind.as_ref().clone() {
        PVar(x) => ctx.stack.bind_expr_decl(x, v),
        PIgnore => {}
        _ => unreachable!(),
    }
}

fn eval_func(ctx: &mut Context, f: Function, vs: Vector<Value>) -> Control {
    match ctx.funcs.get(&(f.x.clone(), f.ts.clone())).cloned() {
        Some(FuncDecl::Def(m, ps, t, b)) => {
            ctx.stack.push_scope(());
            for (p, v) in ps.into_iter().zip(vs) {
                bind(ctx, p, v);
            }
            let c = eval_block(ctx, b);
            ctx.stack.pop_scope();
            match c {
                CValue(v) => CValue(v),
                CReturn(v) => CValue(v),
                CException(s) => CException(s),
                _ => unreachable!(),
            }
        }
        Some(FuncDecl::Bif(m, ts, t)) => eval_builtin(ctx, f.x, f.ts, vs),
        _ => unreachable!("{:?}", f),
    }
}

fn assert_type_params(ts: &Vector<Type>) {
    for t in ts {
        assert!(matches!(t.kind.as_ref(), TVar(..)) == false);
    }
}

fn eval_builtin(ctx: &mut Context, x: Name, ts: Vector<Type>, vs: Vector<Value>) -> Control {
    assert_type_params(&ts);
    let x = x.as_str();
    let ts = ts.into_iter().collect::<Vec<_>>();
    let vs = vs.into_iter().collect::<Vec<_>>();
    let v = ctx.bifs.get(x)(ctx, ts.as_slice(), vs.as_slice());
    CValue(v)
}

fn eval_block(ctx: &mut Context, b: Block) -> Control {
    ctx.stack.push_scope(());
    for s in b.ss {
        match eval_expr_stmt(ctx, s.clone()) {
            CValue(_) => {}
            c @ (CBreak(..) | CContinue(..) | CException(..) | CReturn(..)) => {
                ctx.stack.pop_scope();
                return c;
            }
        }
    }
    let v = var(ctx, b.e);
    ctx.stack.pop_scope();
    CValue(v).into()
}

fn var(ctx: &mut Context, e: Expr) -> Value {
    let EVar(x) = e.kind() else { unreachable!() };
    ctx.find_val(&x)
}

fn eval_expr(ctx: &mut Context, e: Expr) -> Control {
    match e.kind() {
        EConst(c) => CValue(constant(c).into()),
        EFunCallDirect(x, ts, es) => {
            let vs = es.map(|e| var(ctx, e));
            let f = Function { x, ts };
            eval_func(ctx, f, vs)
        }
        EFunCall(e, es) => {
            let f = var(ctx, e).as_function();
            let vs = es.map(|e| var(ctx, e));
            eval_func(ctx, f, vs)
        }
        EDef(x, ts) => CValue(Function { x, ts }.into()).into(),
        EFunReturn(e) => CReturn(var(ctx, e).into()),
        EIfElse(e, b0, b1) => {
            if var(ctx, e).as_bool() {
                eval_block(ctx, b0)
            } else {
                eval_block(ctx, b1)
            }
        }
        ESsaLoop(pes, b) => {
            ctx.stack.push_scope(());
            for (p, e) in pes {
                let v = var(ctx, e);
                bind(ctx, p, v.clone());
            }
            loop {
                match eval_block(ctx, b.clone()) {
                    CValue(_) => {}
                    CException(x) => {
                        ctx.stack.pop_scope();
                        return CException(x);
                    }
                    CBreak(v) => {
                        ctx.stack.pop_scope();
                        return CValue(v);
                    }
                    CContinue(pvs) => {
                        ctx.stack.pop_scope();
                        ctx.stack.push_scope(());
                        for (p, v) in pvs {
                            bind(ctx, p, v);
                        }
                        continue;
                    }
                    CReturn(v) => {
                        ctx.stack.pop_scope();
                        return CReturn(v);
                    }
                }
            }
        }
        EBreak(e) => {
            let v = var(ctx, e);
            CBreak(v)
        }
        ESsaContinue(pes) => {
            let pvs = pes.map(|(p, e)| (p, var(ctx, e)));
            CContinue(pvs)
        }
        ERecord(xes) => {
            let xvs = xes.into_iter().map(|(x, e)| (x, var(ctx, e))).collect();
            CValue(Record(xvs).into())
        }
        ERecordAccess(e, x) => {
            let v = var(ctx, e);
            let xvs = v.as_record();
            CValue(xvs.0.get(&x).unwrap().clone().into())
        }
        EVariant(_, _, x, e) => {
            let v = var(ctx, e);
            CValue(Variant { x, v }.into())
        }
        EVariantAccess(_, _, x, e) => {
            let var = var(ctx, e).as_variant();
            assert_eq!(x, var.x);
            CValue(var.v).into()
        }
        EVariantCheck(_, _, x, e) => {
            let var = var(ctx, e).as_variant();
            CValue((var.x == x).into())
        }
        EArray(es) => {
            let vs = es.map(|e| var(ctx, e));
            CValue(Array(vs).into())
        }
        EArrayAccess(e0, e1) => {
            let v0 = var(ctx, e0).as_array();
            let v1 = var(ctx, e1).as_usize();
            if let Some(v) = v0.0.get(v1) {
                CValue(v.clone().into())
            } else {
                CException(Error::InterpreterError {
                    info: e.info,
                    s: "array index out of bounds".into(),
                })
            }
        }
        ETuple(es) => {
            let vs = es.map(|e| var(ctx, e));
            CValue(Tuple(vs).into())
        }
        ETupleAccess(e, n) => {
            let v = var(ctx, e).as_tuple().0;
            CValue(v[n as usize].clone().into())
        }
        EVar(x) => CValue(var(ctx, e).into()),
        EMatch(_, _) => unreachable!(),
        ERecordConcat(_, _) => todo!(),
        EArrayConcat(_, _) => todo!(),
        EMut(..) => unreachable!(),
        EDo(..) => unreachable!(),
        ENoop(..) => unreachable!(),
        EFor(..) => unreachable!(),
        EWhile(..) => unreachable!(),
        EFun(..) => unreachable!(),
        ELoop(_) => unreachable!(),
        EBreak(_) => unreachable!(),
        EContinue => unreachable!(),
        EError => unreachable!(),
    }
}

pub fn constant(c: Const) -> Value {
    match c {
        CInt(i) => i.into(),
        CBool(b) => b.into(),
        CFloat(f) => (f as f64).into(),
        CString(s) => builtins::string::String::from(s).into(),
        CUnit => ().into(),
        CChar(c) => c.into(),
    }
}

pub use Control::*;
#[derive(Debug, Clone)]
pub enum Control {
    CBreak(Value),
    CContinue(Vector<(Pattern, Value)>),
    CReturn(Value),
    CValue(Value),
    CException(Error),
}
