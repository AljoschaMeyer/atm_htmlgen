use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::{PathBuf};
use std::io;
use std::env;

use sourcefile::SourceFile;
use ropey::{Rope, RopeSlice};
use thiserror::Error;

mod macros;
use macros::*;

mod parse;
use parse::*;

pub struct RunConfiguration {
    pub entrypoint: PathBuf,
    pub output: PathBuf,
}

pub fn run(c: RunConfiguration) {
    if let Err(e) = do_run(c) {
        eprintln!("{}", e);
    }
}

fn do_run(c: RunConfiguration) -> Result<(), YattError> {
    let mut y = Yatt::new(c);

    match std::fs::read_to_string(&y.c.entrypoint) {
        Err(e) => return Err(YattError::EntryIO(e)),
        Ok(entry) => {
            y.source.add_file_raw(&y.c.entrypoint.to_string_lossy(), &entry);
            let ast = parse::parse(&entry, &mut y, 0)?;
            let expanded = macros::expand(ast, &mut y)?;
            let out_string = expanded.to_string();
            match std::fs::write(&y.c.output, &out_string) {
                Err(e) => return Err(YattError::ExitIO(e)),
                Ok(_) => return Ok(()),
            }
        }
    }
}

struct Yatt {
    c: RunConfiguration,
    state: State,
    cwd: PathBuf,
    filename: PathBuf, // within the cwd
    source: SourceFile,
}

impl Yatt {
    fn new(c: RunConfiguration) -> Self {
        let entrypoint = c.entrypoint.clone();
        let cwd = entrypoint.parent().expect("Entrypoint must not be the root of the file system.").to_path_buf();
        let filename = entrypoint.strip_prefix(&cwd).unwrap().to_path_buf();
        Yatt {
            c,
            state: State::new(),
            cwd,
            filename,
            source: SourceFile::new(),
        }
    }
}

#[derive(Error, Debug)]
enum YattError {
    #[error("could not read entrypoint file: {0}")]
    EntryIO(io::Error),
    #[error("could not write primary output file: {0}")]
    ExitIO(io::Error),
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Expansion(#[from] ExpansionError),
}


pub struct State;

impl State {
    fn new() -> Self {
        State
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let c = RunConfiguration {
        entrypoint: args[1].clone().into(),
        output: args[2].clone().into(),
    };
    run(c);
}
