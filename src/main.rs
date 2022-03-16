use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io;
use std::env;
use std::path::PathBuf;

use sourcefile::SourceFile;
use ropey::{Rope, RopeSlice};
use thiserror::Error;

mod macros;
use macros::*;

mod parse;
use parse::*;

pub struct RunConfiguration {
    pub entrypoint: PathBuf,
}

pub fn run(c: RunConfiguration) {
    let mut y = Yatt::new(c).unwrap();

    if let Err(e) = do_run(&mut y) {
        print_yatt_error(e, &y.source);
    }
}

fn do_run(y: &mut Yatt) -> Result<(), YattError> {
    match std::fs::read_to_string(&y.c.entrypoint) {
        Err(e) => return Err(YattError::EntryIO(e)),
        Ok(entry) => {
            y.source.add_file_raw(&y.c.entrypoint.to_string_lossy(), &entry);
            let ast = parse::parse(&entry, y, 0)?;
            let _expanded = macros::expand(ast, y)?;
            return Ok(());
        }
    }
}

struct Yatt {
    c: RunConfiguration,
    pub state: State,
    source: SourceFile,
}

impl Yatt {
    fn new(c: RunConfiguration) -> Result<Self, io::Error> {
        let entrypoint = std::env::current_dir()?.join(c.entrypoint.clone());
        let cwd = std::env::current_dir()?.join(entrypoint.parent().expect("Entrypoint must not be the root of the file system."));
        let filename = entrypoint.strip_prefix(&cwd).unwrap().to_path_buf();
        return Ok(Yatt {
            c,
            state: State::new(cwd, filename),
            source: SourceFile::new(),
        });
    }
}

#[derive(Error, Debug)]
enum YattError {
    #[error("never printed")]
    EntryIO(io::Error),
    #[error("never printed")]
    Parse(#[from] ParseError),
    #[error("never printed")]
    Expansion(#[from] ExpansionError),
}

fn print_yatt_error(e: YattError, source: &SourceFile) {
    println!("Encountered an error, did not produce new output.");

    match e {
        YattError::EntryIO(e) => println!("Failed to read entry file:\n{}", e),
        YattError::Parse(e) => e.print_parse_error(source),
        YattError::Expansion(e) => e.print_expansion_error(source),
    }
}

pub(crate) fn print_trace(t: Trace, source: &SourceFile, show_end: bool) {
    match t {
        Trace(None) => println!("Generated in macro, the macro is buggy and should have thrown error instead"),
        Trace(Some(span)) => {
            let s = source.resolve_offset_span(span.0, span.1).unwrap();
            let offset = if show_end { &s.start } else { &s.end };
            println!("{}, line {}, col {}\n", offset.filename, offset.line, offset.col);
            println!("{}", &source.contents[span.0..span.1]);
        }
    }
}

pub struct State {
    basedir: PathBuf,
    cwd: PathBuf,
    filename: PathBuf, // within the cwd
}

impl State {
    fn new(basedir: PathBuf, initial_file: PathBuf) -> Self {
        State {
            cwd: basedir.clone(),
            basedir,
            filename: initial_file,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let c = RunConfiguration {
        entrypoint: args[1].clone().into(),
    };
    run(c);
}
