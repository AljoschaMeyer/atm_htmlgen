use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::collections::BTreeMap;

use ropey::{Rope, RopeSlice};
use thiserror::Error;

use crate::{State, Yatt};
use crate::parse;
use crate::parse::OffsetSpan;

#[derive(Clone, Debug)]
pub(crate) struct Trace(pub Option<OffsetSpan>);

fn up_and_down<D, U, P>(down: D, up: U, params: &P, args: Box<[OutInternal]>, m_span: Trace, y: &mut Yatt) -> Result<Rope, ExpansionError>
where
    D: Fn(&P, usize /*number of arguments*/, &mut Yatt, Trace) -> Result<Out, ExpansionError>,
    U: Fn(&P, RopeSlice, &mut Yatt, Trace) -> Result<Rope, ExpansionError>,
{
    let o = down(params, args.len(), y, m_span.clone())?;
    let internal = out_to_internal(o, args).map_err(|e| ExpansionError::ArgumentIndex(e, m_span.clone()))?;
    let d = expand(internal, y)?;
    return up(params, d.slice(..), y, m_span);
}

fn up_id<P>(_p: &P, r: RopeSlice, y: &mut Yatt, _trace: Trace) -> Result<Rope, ExpansionError> {
    Ok(r.into())
}

#[derive(Error, Debug)]
pub(crate) enum ExpansionError {
    #[error("TODO {0}")]
    ArgumentIndex(usize, Trace),
    #[error("TODO {0}")]
    ArgumentNumber(usize, Trace),
    #[error("TODO {0}")]
    InputIO(io::Error, Trace),
    #[error("{0}")]
    Parse(#[from] parse::ParseError),
}

type TagParams = BTreeMap<String, String>;

#[derive(Clone)]
pub enum Out {
    Many(Box<[Out]>),
    Argument(usize),
    Text(Rope),
    EmptyMacro((), Box<[Out]>),
    Span(TagParams, Box<[Out]>)
    // Macro {
    //     m: Macro,
    //     arguments: Box<[Out]>,
    // },
    // Input(PathBuf),
}

#[derive(Clone)]
pub(crate) enum OutInternal {
    Many(Box<[OutInternal]>),
    Text(Rope, Trace),
    EmptyMacro(Trace, (), Box<[OutInternal]>),
    Span(Trace, TagParams, Box<[OutInternal]>),
    // Macro {
    //     m: Macro,
    //     m_span: Trace,
    //     arguments: Box<[OutInternal]>,
    // },
    // Input {
    //     span: Trace,
    //     path: PathBuf,
    // },

}

pub(crate) fn expand(out: OutInternal, y: &mut Yatt) -> Result<Rope, ExpansionError> {
    match out {
        OutInternal::Text(r, _) => return Ok(r),

        OutInternal::Many(outs) => {
            let mut r = Rope::new();
            for o in outs.into_iter() {
                r.append(expand(o.clone(), y)?);
            }
            return Ok(r);
        }

        OutInternal::EmptyMacro(trace, _params, args) => {
            if args.len() != 0 {
                return Err(ExpansionError::ArgumentNumber(args.len(), trace));
            } else {
                return Ok(Rope::new());
            }
        }

        // D: Fn(&P, usize /*number of arguments*/, &mut Yatt) -> Result<Out, Box<dyn Error>>,
        // U: Fn(&P, RopeSlice, &mut Yatt) -> Result<Rope, Box<dyn Error>>,
        OutInternal::Span(trace, params, args) => {
            return up_and_down(|p, n, y, trace| {
                html_tag("span", p, n, y, trace)
            }, up_id, &params, args, trace, y);
        }

        // OutInternal::Macro {m, m_span, arguments} => {
        //     match m.0(arguments.len(), &mut y.states) {
        //         Err(e) => return Err(ExpansionError::Down(e, m_span)),
        //         Ok(o) => {
        //             let internal = out_to_internal(o, arguments).map_err(|e| ExpansionError::ArgumentIndex(e, m_span.clone()))?;
        //             let down = expand(internal, y)?;
        //             return m.1(down.slice(..)).map_err(|e| ExpansionError::Up(e, m_span));
        //         }
        //     }
        // }
        //
        // OutInternal::Input {span, path} => {
        //     if path.is_absolute() {
        //         y.cwd = path.parent().expect("Input file must not be the root of the file system.").to_path_buf();
        //         y.filename = path.strip_prefix(&y.cwd).unwrap().to_path_buf();
        //     } else {
        //         let tmp = path.parent().expect("Input file must not be the root of the file system.").to_path_buf();
        //         y.cwd = y.cwd.join(tmp);
        //         y.filename = path.strip_prefix(&y.cwd).unwrap().to_path_buf();
        //     }
        //
        //     match std::fs::read_to_string(&y.cwd.join(&y.filename)) {
        //         Err(e) => return Err(ExpansionError::InputIO(e, span)),
        //         Ok(entry) => {
        //             let ast = parse::parse(&entry, y)?;
        //             return expand(ast, y);
        //         }
        //     }
        // }
    }
}

// Error is the index of an invalid argument.
fn out_to_internal(out: Out, args: Box<[OutInternal]>) -> Result<OutInternal, usize> {
    match out {
        Out::Many(outs) => {
            return Ok(OutInternal::Many(outs_to_internals(outs, args)?));
        }

        Out::Text(t) => return Ok(OutInternal::Text(t, Trace(None))),

        Out::Argument(n) => {
            match args.get(n) {
                None => return Err(n),
                Some(arg) => return Ok(arg.clone()),
            }
        }

        Out::EmptyMacro(params, a) => {
            Ok(OutInternal::EmptyMacro(Trace(None), params, outs_to_internals(a, args)?))
        }

        Out::Span(params, a) => {
            Ok(OutInternal::Span(Trace(None), params, outs_to_internals(a, args)?))
        }

        // Out::Macro { m, arguments: m_args} => {
        //     let mut m_args_internal = Vec::with_capacity(m_args.len());
        //     for arg in m_args.into_iter() {
        //         m_args_internal.push(out_to_internal(arg.clone(), arguments.clone())?);
        //     }
        //     return Ok(OutInternal::Macro {
        //         m,
        //         m_span: Trace(None),
        //         arguments: m_args_internal.into(),
        //     });
        // }
        //
        // Out::Input(path) => return Ok(OutInternal::Input { path, span: Trace(None) }),
    }
}

fn outs_to_internals(outs: Box<[Out]>, args: Box<[OutInternal]>) -> Result<Box<[OutInternal]>, usize> {
    let mut internals = Vec::with_capacity(outs.len());
    for o in outs.into_iter() {
        internals.push(out_to_internal(o.clone(), args.clone())?);
    }
    Ok(internals.into())
}

// D: Fn(&P, usize /*number of arguments*/, &mut Yatt) -> Result<Out, Box<dyn Error>>,
fn html_tag(tag: &str, params: &TagParams, args: usize, _y: &mut Yatt, trace: Trace) -> Result<Out, ExpansionError> {
    if args == 1 {
        let mut open = format!("<{}", tag);
        for (k, v) in params {
            open.push_str(&format!(r###" {}="{}""###, k, v));
        }
        open.push_str(">");

        let close = format!("</{}>", tag);

        return Ok(Out::Many(vec![
            Out::Text(open.into()),
            Out::Argument(0),
            Out::Text(close.into()),
        ].into()));
    } else if args == 2 {
        let mut open = format!("<{}", tag);
        for (k, v) in params {
            if k != "class" {
                open.push_str(&format!(r###" {}="{}""###, k, v));
            }
        }
        open.push_str(r###" class=""###);

        let middle = r###"">"###;

        let close = format!("</{}>", tag);

        return Ok(Out::Many(vec![
            Out::Text(open.into()),
            Out::Argument(0),
            Out::Text(middle.into()),
            Out::Argument(1),
            Out::Text(close.into()),
        ].into()));
    } else if args == 3 {
        let mut open = format!("<{}", tag);
        for (k, v) in params {
            if k != "class" && k != "id" {
                open.push_str(&format!(r###" {}="{}""###, k, v));
            }
        }
        open.push_str(r###" id=""###);

        let start_middle = r###"" class""###;
        let middle = r###"">"###;

        let close = format!("</{}>", tag);

        return Ok(Out::Many(vec![
            Out::Text(open.into()),
            Out::Argument(0),
            Out::Text(start_middle.into()),
            Out::Argument(1),
            Out::Text(middle.into()),
            Out::Argument(2),
            Out::Text(close.into()),
        ].into()));
    } else {
        return Err(ExpansionError::ArgumentNumber(args, trace));
    }
}
