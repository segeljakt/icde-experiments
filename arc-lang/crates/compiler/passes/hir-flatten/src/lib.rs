#![allow(unused)]
pub mod context;

use crate::context::Clause;
use crate::context::Context;
use crate::context::Equations;
use diagnostics::Error;
use hir::*;
use im_rc::ordmap;
use im_rc::vector;
use im_rc::OrdMap;
use im_rc::Vector;
use info::Info;
use utils::OptionUtils;
use utils::VectorUtils;

pub fn process(ctx: &mut Context, ss: Vector<Stmt>) -> Vector<Stmt> {
    for s in ss {
        lower_stmt(ctx, s);
    }
    ctx.take_stmts()
}

fn lower_stmt(ctx: &mut Context, s: Stmt) {
    let info = s.info;
    match s.kind.clone() {
        SDef(m, x, gs, ps, t, b) => {
            ctx.stack.push_scope(Vector::new());
            let ps = ps.map(|p| lower_param(ctx, p));
            let b = lower_block(ctx, b);
            let ss1 = ctx.stack.pop_scope();
            let ss2 = ss1.concat(b.ss);
            let b1 = Block::new(ss2, b.e, info);
            ctx.add_stmt(SDef(m, x, gs, ps, t, b1).with(info))
        }
        SVar(p, e) => {
            let e = lower_expr(ctx, e);
            let p = lower_val_pattern(ctx, p, e);
        }
        SRecDef(..) => todo!(),
        SBif(..) | SEnum(..) | SBit(..) => ctx.add_stmt(s),
        SNoop => {}
        SMonoDef(_, _, _, _, _, _) => unreachable!(),
        SMonoBif(_, _, _, _, _) => unreachable!(),
        SMonoEnum(_, _, _, _) => unreachable!(),
        SMonoBit(_, _, _) => unreachable!(),
    }
}

fn lower_param(ctx: &mut Context, p: Pattern) -> Pattern {
    match p.kind.as_ref().clone() {
        PIgnore => p,
        PNoop(p) => lower_param(ctx, p),
        PVar(_) => p,
        PError => p,
        _ => {
            let info = p.info;
            let t = p.t.clone();
            let x = ctx.vals.fresh();
            let e = EVar(x.clone()).with(p.t.clone(), info);
            lower_val_pattern(ctx, p, e);
            PVar(x).with(t, info)
        }
    }
}

fn lower_val_pattern(ctx: &mut Context, p: Pattern, e: Expr) {
    let t = p.t.clone();
    let info = p.info;
    match p.kind.as_ref().clone() {
        PError | PIgnore => {
            ctx.add_stmt(SVar(p, e).with(info));
        }
        PNoop(p) => lower_val_pattern(ctx, p, e),
        PRecord(xps) => {
            let e = bind_expr(ctx, e);
            for (x, p) in xps {
                let e = ERecordAccess(e.clone(), x).with(p.t.clone(), info);
                lower_val_pattern(ctx, p, e);
            }
        }
        PArray(ps) => {
            let e = bind_expr(ctx, e);
            for (p2, i) in ps.into_iter().zip(0..) {
                let e2 = EConst(CInt(i)).with(TNominal("i32".to_string(), vector![]).into(), info);
                let e = EArrayAccess(e.clone(), e2).with(p2.t.clone(), info);
                lower_val_pattern(ctx, p2, e);
            }
        }
        PArrayConcat(..) => unreachable!(),
        PRecordConcat(..) => unreachable!(),
        PTuple(ps) => {
            let e = bind_expr(ctx, e);
            for (p, i) in ps.into_iter().zip(0..) {
                let e = ETupleAccess(e.clone(), i).with(p.t.clone(), info);
                lower_val_pattern(ctx, p, e);
            }
        }
        PVar(x) => {
            if let EVar(x1) = e.kind.as_ref() {
                ctx.stack.bind_expr_decl(x.clone(), x1.clone())
            } else {
                ctx.add_stmt(SVar(PVar(x).with(t, e.info), e).with(info))
            }
        }
        PConst(_) | POr(_, _) | PVariant(_, _, _, _) => {
            ctx.diagnostics
                .push_error(Error::ExpectedIrrefutablePattern { info });
        }
    }
}

fn bind_expr(ctx: &mut Context, e: Expr) -> Expr {
    if let EVar(x1) = e.kind.as_ref() {
        e
    } else {
        let x = ctx.vals.fresh();
        let p = PVar(x.clone()).with(e.t.clone(), e.info);
        ctx.add_stmt(SVar(p, e.clone()).with(e.info));
        EVar(x).with(e.t.clone(), e.info)
    }
}

fn lower_expr_val(ctx: &mut Context, e: Expr) -> Expr {
    let e = lower_expr(ctx, e);
    match e.kind.as_ref() {
        EVar(x) => e,
        _ => {
            let x = ctx.add_expr(e.clone());
            EVar(x).with(e.t, e.info)
        }
    }
}

fn lower_expr(ctx: &mut Context, e: Expr) -> Expr {
    let info = e.info;
    let t = e.t;
    match e.kind.as_ref().clone() {
        ENoop(e) => lower_expr(ctx, e),
        EBreak(e) => {
            let e = lower_expr_val(ctx, e);
            EBreak(e).with(t, info)
        }
        EFunCall(e, es) => {
            let e = lower_expr_val(ctx, e);
            let es = es.mapm(ctx, lower_expr_val);
            EFunCall(e, es).with(t, info)
        }
        EFunCallDirect(x, ts, es) => {
            let es = es.mapm(ctx, lower_expr_val);
            EFunCallDirect(x, ts, es).with(t, info)
        }
        EContinue => (EContinue.with(t, info)),
        EVariant(x0, ts, x1, e) => {
            let e = lower_expr_val(ctx, e);
            EVariant(x0, ts, x1, e).with(t, info)
        }
        EFun(ts, t1, b) => {
            let b = lower_block(ctx, b);
            EFun(ts, t1, b).with(t, info)
        }
        EConst(c) => (EConst(c).with(t, info)),
        ELoop(b) => {
            let b = lower_block(ctx, b);
            ELoop(b).with(t, info)
        }
        EMatch(e, arms) => {
            let e = lower_expr(ctx, e);
            let x = if let EVar(x) = e.kind.as_ref() {
                ctx.stack.bind_expr_decl(x.clone(), x.clone());
                x.clone()
            } else {
                ctx.add_expr(e.clone())
            };
            let clauses = arms_to_clauses(arms, x);
            let b = lower_clauses(ctx, t, clauses);
            let ctx = ctx.add_stmts(b.ss);
            b.e
        }
        EIfElse(e, b0, b1) => {
            let e = lower_expr_val(ctx, e);
            let b0 = lower_block(ctx, b0);
            let b1 = lower_block(ctx, b1);
            EIfElse(e, b0, b1).with(t, info)
        }
        ERecord(xes) => {
            let xes = xes.map(|(x, e)| (x, lower_expr_val(ctx, e)));
            ERecord(xes).with(t, info)
        }
        ERecordAccess(e, x) => {
            let e = lower_expr_val(ctx, e);
            ERecordAccess(e, x).with(t, info)
        }
        ERecordConcat(_, _) => todo!(),
        EFunReturn(e) => {
            let e = lower_expr_val(ctx, e);
            EFunReturn(e).with(t, info)
        }
        EVar(x) => {
            let x = resolve_alias(ctx, x);
            EVar(x).with(t, info)
        }
        EArray(es) => {
            let es = es.mapm(ctx, lower_expr_val);
            EArray(es).with(t, info)
        }
        EArrayConcat(_, _) => todo!(),
        EArrayAccess(e0, e1) => {
            let e0 = lower_expr_val(ctx, e0);
            let e1 = lower_expr_val(ctx, e1);
            EArrayAccess(e0, e1).with(t, info)
        }
        EMut(e0, e1) => {
            let e0 = lower_expr_val(ctx, e0);
            let e1 = lower_expr_val(ctx, e1);
            EMut(e0, e1).with(t, info)
        }
        EDef(x, ts) => EDef(x, ts).with(t, info),
        EDo(b) => {
            let b = lower_block(ctx, b);
            ctx.add_stmts(b.ss);
            b.e
        }
        ETuple(es) => {
            let es = es.mapm(ctx, lower_expr_val);
            ETuple(es).with(t, info)
        }
        ETupleAccess(e, i) => {
            let e = lower_expr_val(ctx, e);
            ETupleAccess(e, i).with(t, info)
        }
        EError => EError.with(t, info),
        EWhile(_, _) => unreachable!(),
        EFor(_, _, _) => unreachable!(),
        EVariantAccess(_, _, _, _) => unreachable!(),
        EVariantCheck(_, _, _, _) => unreachable!(),
        ESsaLoop(_, _) => unreachable!(),
        ESsaContinue(_) => unreachable!(),
    }
}

fn resolve_alias(ctx: &mut Context, mut x: Name) -> Name {
    for scope in ctx.stack.iter() {
        if let Some(x1) = scope.expr_namespace.find(&x) {
            x = x1.clone();
        }
    }
    x
}

fn simplify_pat_eq(
    ctx: &mut Context,
    or_clauses: &mut Vector<Clause>,
    clause: &mut Clause,
    (v, p): (Name, Pattern),
) {
    match p.kind.as_ref().clone() {
        // Irrefutable patterns can be simplified (i.e., flattened)
        PIgnore => {}
        PNoop(p) => simplify_pat_eq(ctx, or_clauses, clause, (v, p)),
        PRecord(xps) => {
            for (x, p) in xps {
                let e = EVar(v.clone()).with(p.t.clone(), p.info);
                ctx.add_expr(ERecordAccess(e, x).with(p.t.clone(), p.info));
                simplify_pat_eq(ctx, or_clauses, clause, (v.clone(), p))
            }
        }
        // Or-patterns are duplicated for each branch
        POr(p0, p1) => {
            let x0 = ctx.vals.fresh();
            let x1 = ctx.vals.fresh();
            let mut eqs0 = clause.eqs.clone();
            let mut eqs1 = clause.eqs.clone();
            eqs0.insert(x0, p0);
            eqs1.insert(x1, p1);
            let mut c0 = Clause::new(eqs0, clause.substs.clone(), clause.b.clone());
            let mut c1 = Clause::new(eqs1, clause.substs.clone(), clause.b.clone());
            simplify_clause(ctx, or_clauses, &mut c0);
            simplify_clause(ctx, or_clauses, &mut c1);
            or_clauses.push_back(c0);
            or_clauses.push_back(c1);
        }
        PArray(ps) => {
            for (p, i) in ps.into_iter().zip(0..) {
                let e0 = EVar(v.clone()).with(p.t.clone(), p.info);
                let e1 =
                    EConst(CInt(i)).with(TNominal("i32".to_string(), vector![]).into(), p.info);
                ctx.add_expr(EArrayAccess(e0, e1).with(p.t.clone(), p.info));
                simplify_pat_eq(ctx, or_clauses, clause, (v.clone(), p))
            }
        }
        PArrayConcat(_, _) => unreachable!(),
        PRecordConcat(_, _) => unreachable!(),
        // Variables are substituted
        PVar(v1) => {
            clause.substs.insert(v, v1);
        }
        PTuple(ps) => {
            for (p, i) in ps.into_iter().zip(0..) {
                let e = EVar(v.clone()).with(p.t.clone(), p.info);
                ctx.add_expr(ETupleAccess(e, i).with(p.t.clone(), p.info));
                simplify_pat_eq(ctx, or_clauses, clause, (v.clone(), p))
            }
        }
        // Refutable patterns are added to the list of equations
        PConst(_) | PVariant(_, _, _, _) => {
            clause.eqs.insert(v, p);
        }
        PError => {}
    }
}

fn lower_clauses(ctx: &mut Context, match_t: Type, mut clauses: Vector<Clause>) -> Block {
    // Simplify the current clauses
    let mut or_clauses = Vector::new();
    for clause in clauses.iter_mut() {
        simplify_clause(ctx, &mut or_clauses, clause);
    }
    clauses.append(or_clauses);
    // Select one equation in one of the clauses to branch on
    let head = clauses.head().unwrap().clone();
    if head.eqs.is_empty() {
        // This pattern equation is now solved
        for (v0, v1) in head.substs {
            ctx.stack.bind_expr_decl(v0, v1);
        }
        lower_block(ctx, head.b)
    } else {
        let x = most_matching_name(&head.eqs, &clauses);
        let p = head.eqs.get(&x).unwrap();
        match p.kind.as_ref().clone() {
            PConst(c) => branch_const_clauses(ctx, p.info, match_t, clauses, x, c, p.t.clone()),
            PVariant(x0, ts, x1, p1) => {
                branch_variant_clauses(ctx, p.info, match_t, clauses, x, x0, ts, x1, p1.t.clone())
            }
            // Irrefutable patterns are already simplified
            PTuple(_)
            | PVar(_)
            | PArray(_)
            | PIgnore
            | POr(_, _)
            | PNoop(_)
            | PRecord(_)
            | PVar(_)
            | PArrayConcat(_, _)
            | PRecordConcat(_, _)
            | PError => unreachable!(),
        }
    }
}

fn simplify_clause(ctx: &mut Context, cs: &mut Vector<Clause>, c: &mut Clause) {
    for eq in std::mem::take(&mut c.eqs) {
        simplify_pat_eq(ctx, cs, c, eq);
    }
}

fn branch_const_clauses(
    ctx: &mut Context,
    info: Info,
    match_t: Type,
    clauses: Vector<Clause>,
    branch_v: Name,
    c: Const,
    t: Type,
) -> Block {
    let (then_clauses, else_clauses) = split_const_clauses(&branch_v, &c, clauses);
    let e0 = EVar(branch_v.clone()).with(t.clone(), info);
    let e1 = ctx.add_expr_val(EConst(c).with(t.clone(), info));
    match () {
        _ if else_clauses.is_empty() => lower_clauses(ctx, match_t, then_clauses),
        _ if then_clauses.is_empty() => lower_clauses(ctx, match_t, else_clauses),
        _ => {
            let then_b = lower_clauses(ctx, match_t.clone(), then_clauses);
            let else_b = lower_clauses(ctx, match_t.clone(), else_clauses);
            let e2 = ctx.add_expr_val(
                EFunCallDirect(
                    ast::binop!(==).to_string(),
                    vector![t.clone()],
                    vector![e0, e1],
                )
                .with(TNominal("bool".to_string(), vector![]).into(), info),
            );
            let v = ctx.add_expr_val(EIfElse(e2, then_b, else_b).with(match_t, info));
            Block::new(vector![], v, info)
        }
    }
}

// unwrap_v: &Name,
// clause.eqs.insert(unwrap_v.clone(), p1);
fn branch_variant_clauses(
    ctx: &mut Context,
    info: Info,
    match_t: Type,
    clauses: Vector<Clause>,
    branch_v: Name,
    x0: Name,
    ts: Vector<Type>,
    x1: Name,
    t: Type,
) -> Block {
    let (then_clauses, else_clauses) = split_variant_clauses(branch_v.clone(), &x1, clauses);

    match () {
        _ if else_clauses.is_empty() => {
            ctx.stack.push_scope(Vector::new());
            let e0 = EVar(branch_v.clone()).with(t.clone(), info);
            let e1 = EVariantAccess(x0.clone(), ts.clone(), x1.clone(), e0.clone());
            let v = ctx.add_expr(e1.with(t, info));
            let then_clauses = add_equations(then_clauses, v.clone());
            let b = lower_clauses(ctx, match_t, then_clauses);
            let ss1 = ctx.stack.pop_scope();
            let ss2 = ss1.concat(b.ss);
            Block::new(ss2, b.e, info)
        }
        _ if then_clauses.is_empty() => lower_clauses(ctx, match_t, else_clauses),
        _ => {
            let e0 = EVar(branch_v.clone()).with(t.clone(), info);
            let e1 = EVariantAccess(x0.clone(), ts.clone(), x1.clone(), e0.clone());

            let check_e = ctx.add_expr_val(
                EVariantCheck(x0, ts, x1, e0.clone())
                    .with(TNominal("bool".to_string(), vector![]).into(), info),
            );
            let then_b = {
                ctx.stack.push_scope(Vector::new());
                let v = ctx.add_expr(e1.with(t.clone(), info).clone());
                let then_clauses = add_equations(then_clauses, v.clone());
                let b = lower_clauses(ctx, match_t.clone(), then_clauses);
                let ss1 = ctx.stack.pop_scope();
                let ss2 = ss1.concat(b.ss);
                Block::new(ss2, b.e, info)
            };
            let else_b = {
                ctx.stack.push_scope(Vector::new());
                let b = lower_clauses(ctx, match_t.clone(), else_clauses);
                let ss1 = ctx.stack.pop_scope();
                let ss2 = ss1.concat(b.ss);
                Block::new(ss2, b.e, info)
            };
            let e = ctx.add_expr_val(EIfElse(check_e, then_b, else_b).with(match_t, info));
            Block::new(vector![], e, info)
        }
    }
}

fn add_equations(clauses: Vector<(Clause, Option<Pattern>)>, v: Name) -> Vector<Clause> {
    clauses
        .into_iter()
        .map(|(mut c, p)| {
            if let Some(p) = p {
                c.eqs.insert(v.clone(), p);
            }
            c
        })
        .collect()
}

fn split_const_clauses(
    branch_v: &Name,
    branch_c: &Const,
    clauses: Vector<Clause>,
) -> (Vector<Clause>, Vector<Clause>) {
    let mut then_clauses = vector![];
    let mut else_clauses = vector![];
    for mut clause in clauses {
        if let Some(p) = clause.eqs.get(branch_v) {
            if let PConst(c1) = (*p.kind).clone() {
                if *branch_c == c1 {
                    clause.eqs.remove(branch_v);
                    then_clauses.push_back(clause);
                } else {
                    else_clauses.push_back(clause);
                }
            } else {
                unreachable!()
            }
        } else {
            then_clauses.push_back(clause.clone());
            else_clauses.push_back(clause);
        }
    }
    (then_clauses, else_clauses)
}

fn split_variant_clauses(
    branch_v: Name,
    variant_x: &Name,
    clauses: Vector<Clause>,
) -> (Vector<(Clause, Option<Pattern>)>, Vector<Clause>) {
    let mut then_clauses = vector![];
    let mut else_clauses = vector![];
    for mut clause in clauses {
        if let Some(p) = clause.eqs.get(&branch_v) {
            if let PVariant(_xs, _ts, x1, p1) = (*p.kind).clone() {
                if *variant_x == x1 {
                    clause.eqs.remove(&branch_v);
                    then_clauses.push_back((clause, Some(p1)));
                } else {
                    else_clauses.push_back(clause);
                }
            } else {
                unreachable!()
            }
        } else {
            then_clauses.push_back((clause.clone(), None));
            else_clauses.push_back(clause);
        }
    }
    (then_clauses, else_clauses)
}

// Pick the variable which occurs in the most equations of all clauses
fn most_matching_name(eqs: &Equations, clauses: &Vector<Clause>) -> Name {
    eqs.iter()
        .map(|(x, _)| x)
        .max_by_key(|x| {
            clauses
                .iter()
                .filter(|clause| clause.eqs.iter().find(|(y, _)| x == y).is_some())
                .count()
        })
        .cloned()
        .unwrap()
}

fn lower_block(ctx: &mut Context, b: Block) -> Block {
    ctx.stack.push_scope(Vector::new());
    for s in b.ss {
        lower_stmt(ctx, s);
    }
    let e = lower_expr_val(ctx, b.e.clone());
    let ss = ctx.stack.pop_scope();
    Block::new(ss, e, b.info)
}

fn arms_to_clauses(arms: Vector<Arm>, x: Name) -> Vector<Clause> {
    arms.into_iter()
        .map(|(p, b)| Clause::new(ordmap![x.clone() => p], ordmap![], b))
        .collect()
}
