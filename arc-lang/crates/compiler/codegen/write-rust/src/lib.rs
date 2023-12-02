#![allow(unused)]

use builtins::aggregator::Aggregator;
use builtins::assigner::Assigner;
use builtins::duration::Duration;
use builtins::encoding::Encoding;
use builtins::path::Path;
use builtins::reader::Reader;
use builtins::socket::SocketAddr;
use builtins::time_source::TimeSource;
use builtins::writer::Writer;
use codegen::*;
use std::io::Result;
use std::io::Write;
use value::dynamic::Dataflow;
use value::dynamic::Function;
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

use rust::*;

use im_rc::Vector;

pub fn write(ctx: &mut Context<impl Write>, ss: &Vector<Item>) -> Result<()> {
    ctx.keyword("use")?;
    ctx.space()?;
    ctx.lit("runtime::prelude::*;")?;
    ctx.newline()?;
    ctx.each(ss.iter(), write_item)
}

pub fn write_dataflow(ctx: &mut Context<impl Write>, d: &Dataflow) -> Result<()> {
    let cwd = std::env::current_dir()?;
    ctx.keyword("fn")?;
    ctx.space()?;
    ctx.def("main")?;
    ctx.paren(|_| Ok(()))?;
    ctx.space()?;
    ctx.block(|ctx| {
        ctx.fmt(format_args!("std::env::set_current_dir({cwd:?}).unwrap();"))?;
        ctx.newline()?;
        ctx.lit(r#"let db = Database::new(concat!(env!("CARGO_MANIFEST_DIR"), "/db"));"#)?;
        ctx.newline()?;
        ctx.lit(r#"Runner::new(concat!(env!("CARGO_MANIFEST_DIR"), "/log")).spawn(instance(db.clone()));"#)
    })?;
    ctx.newline()?;
    ctx.keyword("async")?;
    ctx.space()?;
    ctx.keyword("fn")?;
    ctx.space()?;
    ctx.def("instance")?;
    ctx.paren(|ctx| ctx.lit("_db: Database"))?;
    ctx.space()?;
    ctx.block(|ctx| {
        d.streams.iter().try_for_each(|n| write_stream(ctx, n))?;
        d.sinks.iter().try_for_each(|n| write_sink(ctx, n))?;
        Ok(())
    })?;
    Ok(())
}

pub fn write_sink(ctx: &mut Context<impl Write>, s: &Sink) -> Result<()> {
    let (x, w, e) = s.0.as_ref();
    ctx.def("Stream::sink")?;
    ctx.paren(|ctx| {
        ctx.val(&x)?;
        ctx.comma()?;
        write_writer(ctx, w)?;
        ctx.comma()?;
        write_encoding(ctx, e)?;
        Ok(())
    })
}

pub fn write_stream(ctx: &mut Context<impl Write>, s: &Stream) -> Result<()> {
    ctx.newline()?;
    ctx.keyword("let")?;
    ctx.space()?;
    ctx.val(&s.name)?;
    ctx.lit(" = ")?;
    match s.kind.as_ref() {
        DSource(a0, a1, a2) => {
            ctx.def("Stream::source")?;
            ctx.paren(|ctx| {
                write_reader(ctx, a0)?;
                ctx.comma()?;
                write_encoding(ctx, a1)?;
                ctx.comma()?;
                write_time_source(ctx, a2)
            })?;
        }
        DMap(a0, a1) => {
            ctx.def("Stream::map")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_function(ctx, a1)
            })?;
        }
        DFilter(a0, a1) => {
            ctx.def("Stream::filter")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_function(ctx, a1)
            })?;
        }
        DApply(a0, a1) => {
            ctx.def("Stream::apply")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_function(ctx, a1)
            })?;
        }
        DMerge(a0) => {
            ctx.def("Stream::merge")?;
            ctx.paren(|ctx| ctx.seq(a0, |ctx, s| ctx.val(&s)))?;
        }
        DKeyby(a0, a1) => {
            ctx.def("Stream::keyby")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_function(ctx, a1)
            })?;
        }
        DWindow(a0, a1, a2) => {
            ctx.def("Stream::window")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_discretizer(ctx, a1)?;
                ctx.comma()?;
                write_aggregator(ctx, a2)
            })?;
        }
        DFlatten(a0) => {
            ctx.def("Stream::flatten")?;
            ctx.paren(|ctx| ctx.val(&a0))?;
        }
        DFlatMap(a0, a1) => {
            ctx.def("Stream::flat_map")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_function(ctx, a1)
            })?;
        }
        DScan(a0, a1) => {
            ctx.def("Stream::scan")?;
            ctx.paren(|ctx| {
                ctx.val(&a0)?;
                ctx.comma()?;
                write_function(ctx, a1)
            })?;
        }
        DUnkey(a0) => {
            ctx.def("Stream::unkey")?;
            ctx.paren(|ctx| ctx.val(&a0))?;
        }
        DFlatten(x) => {
            ctx.def("Stream::flatten")?;
            ctx.paren(|ctx| ctx.val(&x))?;
        }
    }
    ctx.lit(";")?;
    ctx.newline()?;
    Ok(())
}

fn write_aggregator(
    ctx: &mut Context<impl Write>,
    a: &Aggregator<Function, Function, Function, Function>,
) -> Result<()> {
    match a {
        Aggregator::Incremental {
            lift,
            combine,
            lower,
        } => {
            ctx.def("Aggregator")?;
            ctx.lit("::")?;
            ctx.def("incremental")?;
            ctx.paren(|ctx| {
                write_function(ctx, lift)?;
                ctx.comma()?;
                write_function(ctx, combine)?;
                ctx.comma()?;
                write_function(ctx, lower)
            })
        }
        Aggregator::Holistic { compute } => {
            ctx.def("Aggregator")?;
            ctx.lit("::")?;
            ctx.def("holistic")?;
            ctx.paren(|ctx| write_function(ctx, compute))
        }
    }
}

fn write_discretizer(ctx: &mut Context<impl Write>, d: &Assigner) -> Result<()> {
    match d {
        Assigner::Tumbling { length } => {
            ctx.def("Discretizer")?;
            ctx.lit("::")?;
            ctx.def("tumbling")?;
            ctx.paren(|ctx| ctx.dbg(length))
        }
        Assigner::Sliding { length, step } => {
            ctx.def("Discretizer")?;
            ctx.lit("::")?;
            ctx.def("sliding")?;
            ctx.paren(|ctx| {
                ctx.dbg(length)?;
                ctx.comma()?;
                write_duration(ctx, step)
            })
        }
        Assigner::Session { gap } => {
            ctx.def("Discretizer")?;
            ctx.lit("::")?;
            ctx.def("session")?;
            ctx.paren(|ctx| write_duration(ctx, gap))
        }
        Assigner::Counting { length } => {
            ctx.def("Discretizer")?;
            ctx.lit("::")?;
            ctx.def("counting")?;
            ctx.paren(|ctx| ctx.lit(length))
        }
        Assigner::Moving { length, step } => {
            ctx.def("Discretizer")?;
            ctx.lit("::")?;
            ctx.def("moving")?;
            ctx.paren(|ctx| {
                ctx.lit(length)?;
                ctx.comma()?;
                ctx.lit(step)
            })
        }
    }
}

fn write_duration(ctx: &mut Context<impl Write>, d: &Duration) -> Result<()> {
    ctx.lit("Duration")?;
    ctx.lit("::")?;
    ctx.lit("from_seconds")?;
    ctx.paren(|ctx| ctx.lit(d.0.whole_seconds()))
}

fn write_function(ctx: &mut Context<impl Write>, f: &Function) -> Result<()> {
    ctx.def(&f.x)
}

fn write_time_source(ctx: &mut Context<impl Write>, e: &TimeSource<Function>) -> Result<()> {
    match e {
        TimeSource::Ingestion { watermark_interval } => {
            ctx.def("TimeSource")?;
            ctx.lit("::")?;
            ctx.def("ingestion")?;
            ctx.paren(|ctx| write_duration(ctx, watermark_interval))
        }
        TimeSource::Event {
            extractor,
            watermark_interval,
            slack,
        } => {
            ctx.def("TimeSource")?;
            ctx.lit("::")?;
            ctx.def("event")?;
            ctx.paren(|ctx| {
                write_function(ctx, extractor)?;
                ctx.comma()?;
                write_duration(ctx, watermark_interval)?;
                ctx.comma()?;
                write_duration(ctx, slack)
            })
        }
    }
}

pub fn write_reader(ctx: &mut Context<impl Write>, r: &Reader) -> Result<()> {
    match r {
        Reader::Stdin => {
            ctx.def("Reader")?;
            ctx.lit("::")?;
            ctx.def("stdin")?;
            ctx.paren(|_| Ok(()))?;
        }
        Reader::File { path, watch } => {
            ctx.def("Reader")?;
            ctx.lit("::")?;
            ctx.def("file")?;
            ctx.paren(|ctx| {
                write_path(ctx, path)?;
                ctx.comma()?;
                ctx.dbg(watch)
            })?;
        }
        Reader::Http { url } => {
            ctx.def("Reader")?;
            ctx.lit("::")?;
            ctx.def("http")?;
            ctx.paren(|ctx| ctx.dbg(url))?;
        }
        Reader::Tcp { addr } => {
            ctx.def("Reader")?;
            ctx.lit("::")?;
            ctx.def("tcp")?;
            ctx.paren(|ctx| write_socket_addr(ctx, addr))?;
        }
        Reader::Kafka { addr, topic } => {
            ctx.def("Reader")?;
            ctx.lit("::")?;
            ctx.def("kafka")?;
            ctx.paren(|ctx| {
                ctx.dbg(addr)?;
                ctx.comma()?;
                ctx.dbg(topic)
            })?;
        }
    }
    Ok(())
}

pub fn write_writer(ctx: &mut Context<impl Write>, w: &Writer) -> Result<()> {
    match w {
        Writer::Stdout => {
            ctx.def("Writer::stdout")?;
            ctx.paren(|_| Ok(()))?;
        }
        Writer::File { path } => {
            ctx.def("Writer::file")?;
            ctx.paren(|ctx| write_path(ctx, path))?;
        }
        Writer::Http { url } => {
            ctx.def("Writer::http")?;
            ctx.paren(|ctx| ctx.dbg(url))?;
        }
        Writer::Tcp { addr } => {
            ctx.def("Writer::tcp")?;
            ctx.paren(|ctx| write_socket_addr(ctx, addr))?;
        }
        Writer::Kafka { addr, topic } => {
            ctx.def("Writer::kafka")?;
            ctx.paren(|ctx| {
                ctx.dbg(addr)?;
                ctx.comma()?;
                ctx.lit(topic)
            })?;
        }
    }
    Ok(())
}

pub fn write_path(ctx: &mut Context<impl Write>, p: &Path) -> Result<()> {
    ctx.def("Path::new")?;
    ctx.paren(|ctx| ctx.dbg(&p.0))
}

pub fn write_socket_addr(ctx: &mut Context<impl Write>, s: &SocketAddr) -> Result<()> {
    ctx.def("SocketAddr::new")?;
    ctx.paren(|ctx| {
        ctx.quote(|ctx| ctx.dbg(s.0.ip()))?;
        ctx.comma()?;
        ctx.dbg(s.0.port())
    })
}

pub fn write_encoding(ctx: &mut Context<impl Write>, e: &Encoding) -> Result<()> {
    match e {
        Encoding::Json => {
            ctx.def("Encoding::json")?;
            ctx.paren(|_| Ok(()))?;
        }
        Encoding::Csv { sep } => {
            ctx.def("Encoding::csv")?;
            ctx.paren(|ctx| ctx.dbg(sep))?;
        }
    }
    Ok(())
}

pub fn write_item(ctx: &mut Context<impl Write>, i: &Item) -> Result<()> {
    match &i.kind {
        IDef(m, x, vs, t, b) => {
            ctx.keyword("fn")?;
            ctx.space()?;
            ctx.lit(x)?;
            write_params(ctx, vs)?;
            ctx.lit(" -> ")?;
            write_type(ctx, t)?;
            ctx.space()?;
            write_block(ctx, b)?;
            ctx.newline()?;
        }
        IEnum(m, x, xts) => {
            ctx.mac("#[data]")?;
            ctx.keyword("enum")?;
            ctx.space()?;
            ctx.lit(x)?;
            ctx.space()?;
            ctx.brace(|ctx| {
                ctx.indented_comma_seq(xts, |ctx, (x, t)| {
                    ctx.lit(x)?;
                    write_type(ctx, t)
                })
            })?;
            ctx.newline()?;
        }
        IStruct(x, xts) => {
            ctx.mac("#[data]")?;
            ctx.keyword("struct")?;
            ctx.space()?;
            ctx.lit(x)?;
            ctx.space()?;
            ctx.brace(|ctx| {
                ctx.indented_comma_seq(xts, |ctx, (x, t)| {
                    ctx.lit(x)?;
                    ctx.lit(":")?;
                    write_type(ctx, t)
                })
            })?;
            ctx.newline()?;
        }
        IError => unreachable!(),
    }
    Ok(())
}

pub fn write_params(ctx: &mut Context<impl Write>, ps: &Vector<Val>) -> Result<()> {
    ctx.paren(|ctx| {
        ctx.seq(ps, |ctx, v| {
            write_val(ctx, v)?;
            ctx.lit(":")?;
            ctx.space()?;
            write_type(ctx, &v.t)
        })
    })?;
    Ok(())
}

pub fn write_stmt(ctx: &mut Context<impl Write>, s: &Stmt) -> Result<()> {
    if !s.vs.is_empty() {
        ctx.keyword("let")?;
        ctx.space()?;
        ctx.seq(&s.vs, write_val)?;
        ctx.lit(" = ")?;
    }
    match &s.kind {
        SStruct(x, xvs) => {
            ctx.lit(x)?;
            ctx.space()?;
            ctx.brace(|ctx| {
                ctx.seq(xvs, |ctx, (x, v)| {
                    ctx.lit(x)?;
                    ctx.lit(":")?;
                    write_val(ctx, &v)
                })
            })?;
        }
        SStructAccess(v, x) => {
            write_val(ctx, v)?;
            ctx.lit(".")?;
            ctx.lit(x)?;
        }
        SLoop(vvs, b) => {
            for (v0, v1) in vvs {
                ctx.keyword("let")?;
                ctx.space()?;
                ctx.keyword("mut")?;
                write_val(ctx, v0)?;
                ctx.lit(" = ")?;
                write_val(ctx, v1)?;
                ctx.lit(";")?;
                ctx.newline()?;
            }
            ctx.keyword("loop")?;
            ctx.space()?;
        }
        SBreak(v) => {
            ctx.keyword("break")?;
            ctx.space()?;
            write_val(ctx, v)?;
        }
        SContinue(vvs) => {
            for (v0, v1) in vvs {
                write_val(ctx, v0)?;
                ctx.lit(" = ")?;
                write_val(ctx, v1)?;
                ctx.lit(";")?;
                ctx.newline()?;
            }
            ctx.keyword("continue")?;
        }
        SFunCallDirect(x, vs) => {
            ctx.def(x)?;
            ctx.paren(|ctx| ctx.seq(vs, write_val))?;
        }
        SFunCallIndirect(v, vs) => {
            write_val(ctx, v)?;
            ctx.paren(|ctx| ctx.seq(vs, write_val))?;
        }
        SVariant(x, v) => {
            ctx.lit(x)?;
            ctx.paren(|ctx| write_val(ctx, v))?;
        }
        SVariantCheck(x, v) => {
            ctx.mac("matches!")?;
            ctx.paren(|ctx| {
                write_val(ctx, v)?;
                ctx.comma()?;
                ctx.lit(x)?;
                ctx.paren(|ctx| ctx.lit("_"))
            })?;
        }
        SVariantAccess(x, v) => {
            ctx.mac("unwrap!")?;
            ctx.paren(|ctx| {
                write_val(ctx, v)?;
                ctx.comma()?;
                ctx.lit(x)
            })?;
        }
        SFun(x) => {
            ctx.def(x)?;
        }
        SConst(c) => match c {
            CInt(c) => {
                ctx.lit(c)?;
            }
            CFloat(c) => {
                ctx.lit(c)?;
            }
            CString(c) => {
                ctx.text(&format!(r#""{c}""#))?;
            }
            CBool(c) => {
                ctx.fmt(format_args!("{c}"))?;
            }
            CChar(c) => {
                ctx.text(&format!("'{c}'"))?;
            }
            CUnit => {
                ctx.lit("()")?;
            }
        },
        SIfElse(v, b0, b1) => {
            ctx.keyword("if")?;
            ctx.space()?;
            write_val(ctx, v)?;
            ctx.space()?;
            write_block(ctx, b0)?;
            ctx.space()?;
            ctx.keyword("else")?;
            ctx.space()?;
            write_block(ctx, b1)?;
        }
        SFunReturn(v) => {
            ctx.keyword("return")?;
            ctx.space()?;
            write_val(ctx, v)?;
        }
        SBlockResult(v) => {
            write_val(ctx, v)?;
        }
        SArray(es) => {
            ctx.ty("Array")?;
            ctx.lit("::")?;
            ctx.def("new")?;
            ctx.paren(|ctx| ctx.brack(|ctx| ctx.seq(es, write_val)))?;
        }
        SError => unreachable!(),
    }
    ctx.lit(";")?;
    Ok(())
}

pub fn write_block(ctx: &mut Context<impl Write>, b: &Block) -> Result<()> {
    ctx.block(|ctx| {
        for s in &b.ss {
            ctx.newline()?;
            write_stmt(ctx, s)?;
        }
        Ok(())
    })
}

pub fn write_type(ctx: &mut Context<impl Write>, t: &Type) -> Result<()> {
    match t.kind.as_ref() {
        TFun(ts, t) => {
            ctx.keyword("fn")?;
            ctx.paren(|ctx| ctx.seq(ts, write_type))?;
            ctx.lit(" -> ")?;
            write_type(ctx, t)?;
        }
        TNominal(x, ts) => {
            ctx.ty(x)?;
            if !ts.is_empty() {
                ctx.angle(|ctx| ctx.seq(ts, write_type))?;
            }
        }
        TError => unreachable!(),
    }
    Ok(())
}

pub fn write_val(ctx: &mut Context<impl Write>, v: &Val) -> Result<()> {
    match &v.kind {
        VName(x) => ctx.lit(&x),
        VIgnore => ctx.lit("_"),
        VError => unreachable!(),
    }
}

pub fn write_typeof(ctx: &mut Context<impl Write>, v: &Val) -> Result<()> {
    write_type(ctx, &v.t)
}
