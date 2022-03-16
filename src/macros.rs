use sourcefile::SourceFile;
use std::error::Error;
use std::io;
use std::fs;
use std::collections::BTreeMap;
use std::path::PathBuf;

use ropey::{Rope, RopeSlice};
use thiserror::Error;

use crate::{State, Yatt, print_trace};
use crate::parse;
use crate::parse::OffsetSpan;

#[derive(Clone, Debug)]
pub(crate) struct Trace(pub Option<OffsetSpan>);

fn down_macro<D, P>(down: D, params: &P, args: Vec<OutInternal>, m_span: Trace, y: &mut Yatt) -> Result<Rope, ExpansionError>
where
    D: Fn(&P, usize /*number of arguments*/, &mut Yatt, Trace) -> Result<Out, ExpansionError>,
{
    let o = down(params, args.len(), y, m_span.clone())?;
    let internal = out_to_internal(o, args).map_err(|e| ExpansionError::ArgumentIndex(e, m_span.clone()))?;
    let d = expand(internal, y)?;
    return Ok(d);
}

fn up_macro<U, P>(up: U, params: &P, args: Vec<OutInternal>, m_span: Trace, y: &mut Yatt) -> Result<Rope, ExpansionError>
where
    U: Fn(&P, &[Rope], &mut Yatt, Trace) -> Result<Rope, ExpansionError>,
{
    let mut expanded_args = Vec::new();
    for arg in args {
        expanded_args.push(expand(arg, y)?);
    }

    return up(params, &expanded_args, y, m_span);
}

#[derive(Error, Debug)]
pub(crate) enum ExpansionError {
    #[error("never printed")]
    ArgumentIndex(usize, Trace),
    #[error("never printed")]
    ArgumentNumber(usize, Trace),
    #[error("never printed")]
    InputIO(io::Error, PathBuf, Trace),
    #[error("never printed")]
    OutputIO(io::Error, PathBuf, Trace),
    #[error("never printed")]
    CopyAll(fs_extra::error::Error, PathBuf, PathBuf, Trace),
    #[error("never printed")]
    Parse(#[from] parse::ParseError),
}

impl ExpansionError {
    pub fn print_expansion_error(&self, source: &SourceFile) {
        match self {
            ExpansionError::ArgumentIndex(i, t) => {
                println!("Buggy macro referred to argument number {}, but it did not get that many arguments.", i);
                print_trace(t.clone(), source, false);
            }
            ExpansionError::ArgumentNumber(i, t) => {
                println!("Macro received an invalid number of arguments ({}).", i);
                print_trace(t.clone(), source, false);
            }
            ExpansionError::InputIO(e, path, t) => {
                println!("Failed to read input file {}:\n {}\n", path.to_string_lossy(), e);
                print_trace(t.clone(), source, false);
            }
            ExpansionError::OutputIO(e, path, t) => {
                println!("Failed to write output file {}:\n {}\n", path.to_string_lossy(), e);
                print_trace(t.clone(), source, false);
            }
            ExpansionError::CopyAll(e, from, to, t) => {
                println!("Failed to copy file {} to {}:\n {}\n", from.to_string_lossy(), to.to_string_lossy(), e);
                print_trace(t.clone(), source, false);
            }
            ExpansionError::Parse(e) => {
                e.print_parse_error(source);
            }
        }
    }
}

type TagParams = BTreeMap<String, String>;

#[derive(Clone)]
pub enum Out {
    Many(Vec<Out>),
    Argument(usize),
    Text(Rope),
    HtmlTag(String, TagParams, Vec<Out>),
    Input([PathBuf; 1], Vec<Out>),
    Output([PathBuf; 1], Vec<Out>, bool),
}

#[derive(Clone)]
pub(crate) enum OutInternal {
    Many(Vec<OutInternal>),
    Text(Rope, Trace),
    EmptyMacro(Trace, (), Vec<OutInternal>),
    HtmlTag(Trace, String, TagParams, Vec<OutInternal>),
    Input(Trace, [PathBuf; 1], Vec<OutInternal>),
    Output(Trace, [PathBuf; 1], Vec<OutInternal>, bool),
    CopyAll(Trace, [PathBuf; 2], Vec<OutInternal>),
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

        OutInternal::HtmlTag(trace, tag, params, args) => {
            return down_macro(|p, n, y, trace| {
                html_tag(&tag, p, n, y, trace)
            }, &params, args, trace, y);
        }

        OutInternal::Input(span, path, args) => {
            arguments_exact(0, &args, &span)?;
            let path = &path[0];
            if path.is_absolute() {
                let tmp = path.parent().expect("Input file must not be the root of the file system.").to_path_buf();
                y.state.cwd = y.state.basedir.join(&tmp.strip_prefix("/").unwrap());
                y.state.filename = path.strip_prefix(&tmp).unwrap().to_path_buf();
            } else {
                let tmp = path.parent().expect("Input file must not be the root of the file system.").to_path_buf();
                y.state.cwd = y.state.cwd.join(&tmp);
                y.state.filename = path.strip_prefix(&tmp).unwrap().to_path_buf();
            }

            let p = y.state.cwd.join(&y.state.filename);

            match fs::read_to_string(&p) {
                Err(e) => return Err(ExpansionError::InputIO(e, p, span)),
                Ok(entry) => {
                    let source_offset = y.source.contents.len();
                    y.source.add_file_raw(&y.c.entrypoint.to_string_lossy(), &entry);
                    let ast = parse::parse(&entry, y, source_offset)?;
                    return expand(ast, y);
                }
            }
        }

        OutInternal::Output(span, path, args, tee) => {
            arguments_exact(1, &args, &span)?;
            return up_macro(|path, args, y, span| {
                let path = &path[0];
                let content = args[0].to_string();

                let p = if path.is_absolute() {
                    y.state.basedir.join(&path.strip_prefix("/").unwrap())
                } else {
                    y.state.cwd.join(path)
                };

                let mut dirname = p.clone();
                dirname.pop();
                let _ = fs_extra::dir::create_all(dirname, false);

                match fs::write(&p, &content) {
                    Err(e) => return Err(ExpansionError::OutputIO(e, p, span)),
                    Ok(()) => {
                        if tee {
                            return Ok(args[0].clone());
                        } else {
                            return Ok(Rope::new());
                        }
                    }
                }
            }, &path, args, span, y);
        }

        OutInternal::CopyAll(span, params, args) => {
            arguments_exact(0, &args, &span)?;
            let from = &params[0];
            let to = &params[1];

            fs_extra::copy_items(&params[0..1], to, &fs_extra::dir::CopyOptions::new())
                .map_err(|e| ExpansionError::CopyAll(e, from.clone(), to.clone(), span))?;

            return Ok(Rope::new());
        }
    }
}

fn arguments_exact(n: usize, args: &[OutInternal], span: &Trace) -> Result<(), ExpansionError> {
    if args.len() != n {
        return Err(ExpansionError::ArgumentNumber(args.len(), span.clone()));
    } else {
        return Ok(());
    }
}

// Error is the index of an invalid argument.
fn out_to_internal(out: Out, args: Vec<OutInternal>) -> Result<OutInternal, usize> {
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

        Out::HtmlTag(tag, params, a) => {
            Ok(OutInternal::HtmlTag(Trace(None), tag, params, outs_to_internals(a, args)?))
        }

        Out::Input(path, a) => return Ok(OutInternal::Input(Trace(None), path, outs_to_internals(a, args)?)),

        Out::Output(path, a, tee) => return Ok(OutInternal::Output(Trace(None), path, outs_to_internals(a, args)?, tee)),
    }
}

fn outs_to_internals(outs: Vec<Out>, args: Vec<OutInternal>) -> Result<Vec<OutInternal>, usize> {
    let mut internals = Vec::with_capacity(outs.len());
    for o in outs.into_iter() {
        internals.push(out_to_internal(o.clone(), args.clone())?);
    }
    Ok(internals.into())
}

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
