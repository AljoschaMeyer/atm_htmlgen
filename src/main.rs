use std::collections::HashSet;
use std::path::Path;
use std::collections::HashMap;
use std::io;
use std::env;
use std::path::PathBuf;

use sourcefile::SourceFile;
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
    match std::fs::read_to_string(&y.state.entrypoint) {
        Err(e) => return Err(YattError::EntryIO(e)),
        Ok(entry) => {
            y.source.add_file_raw(&y.state.entrypoint.to_string_lossy(), &entry);
            let ast = parse::parse(&entry, y, 0)?;
            let _expanded = macros::expand(ast, y)?;

            if y.state.second_iteration {
                return Ok(());
            } else {
                let sticky_state = y.state.sticky_state.clone();
                y.state = State::new(y.state.entrypoint.clone())?;
                y.state.second_iteration = true;
                y.state.sticky_state = sticky_state;
                return do_run(y);
            }
        }
    }
}

struct Yatt {
    pub state: State,
    source: SourceFile,
}

impl Yatt {
    fn new(c: RunConfiguration) -> Result<Self, io::Error> {
        return Ok(Yatt {
            state: State::new(c.entrypoint)?,
            source: SourceFile::new(),
        });
    }
}

#[derive(Error, Debug)]
enum YattError {
    #[error("never printed")]
    EntryIO(#[from] io::Error),
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
        Trace(None) => println!("Generated in macro at unknown location."),
        Trace(Some(span)) => {
            let s = source.resolve_offset_span(span.0, span.1).unwrap();
            let offset = if show_end { &s.start } else { &s.end };
            println!("{}, line {}, col {}\n", offset.filename, offset.line, offset.col);
            println!("{}", &source.contents[span.0..span.1]);
        }
    }
}

pub(crate) struct State {
    pub entrypoint: PathBuf,
    pub current_file: PathBuf,
    pub current_output: PathBuf,
    pub second_iteration: bool,
    pub sticky_state: StickyState,

    pub domain: String,

    pub hsection_level: usize,
    pub hsection_current_count: [usize; 6],
    pub hsection_pre_number: [String; 6],
    pub hsection_post_number: [String; 6],
    pub hsection_render_number: [bool; 6],
    pub hsection_name: [String; 6],

    pub box_exercise_current_count: usize,
    pub box_exercise_level: usize,
    pub box_other_current_count: usize,
    pub box_other_level: usize,
    pub box_current: Option<String>, // id of current box if any

    pub box_previews: HashSet<String>,
    pub boxless_previews: HashSet<String>,
}

impl State {
    fn new(entrypoint: PathBuf) -> Result<Self, io::Error> {
        let entrypoint = std::env::current_dir()?.join(entrypoint.clone());
        let cwd = entrypoint.parent().expect("entrypoint must not be the root of the file system.").to_path_buf();
        std::env::set_current_dir(&cwd)?;

        return Ok(State {
            current_file: entrypoint.clone(),
            entrypoint,
            current_output: "".into(),
            second_iteration: false,
            sticky_state: StickyState::new(),

            domain: "http://localhost:8080/".to_string(),

            hsection_level: 0,
            hsection_current_count: [0; 6],
            hsection_pre_number: ["".into(), "<div>Chapter ".into(), "".into(), "".into(), "".into(), "".into()],
            hsection_post_number: ["".into(), "</div>".into(), ": ".into(),": ".into(), ": ".into(), ": ".into()],
            hsection_render_number: [false, true, true, false, false, false],
            hsection_name: ["".into(), "Chapter".into(), "Section".into(), "Subsection".into(), "Subsubsection".into(), "Subsubsubsection".into()],

            box_exercise_current_count: 0,
            box_exercise_level: 1,
            box_other_current_count: 0,
            box_other_level: 1,
            box_current: None,

            box_previews: HashSet::new(),
            boxless_previews: HashSet::new(),
        });
    }

    pub(crate) fn cwd(&self) -> PathBuf {
        self.current_file.parent().expect("Cwd must not be the root of the file system.").to_path_buf()
    }

    // pub(crate) fn current_filename(&self) -> PathBuf {
    //     self.current_file.strip_prefix(&self.cwd()).unwrap().to_path_buf()
    // }

    pub(crate) fn base_dir(&self) -> PathBuf {
        self.entrypoint.parent().expect("Entrypoint must not be the root of the file system.").to_path_buf()
    }

    pub(crate) fn current_output_relative(&self) -> PathBuf {
        self.current_output.strip_prefix(self.base_dir().join("build/")).unwrap().to_path_buf()
    }

    // pub(crate) fn relative_path(&self, to: &Path) -> PathBuf {
    //     let mut cwd_relative = self.cwd().strip_prefix(self.base_dir()).unwrap().to_path_buf();
    //     let up = cwd_relative.components().count();
    //     if up == 0 {
    //         return self.current_output_relative();
    //     } else {
    //         for _ in 0..up {
    //             cwd_relative.push("..");
    //         }
    //         return cwd_relative.join(self.current_output_relative());
    //     }
    // }
    //
    // pub(crate) fn relative_link(&self, id: &str) -> PathBuf {
    //     let info = self.sticky_state.ids.get(id).unwrap();
    //     return self.relative_path(&info.file);
    // }

    pub(crate) fn register_id(&mut self, id: impl Into<String>, kind: CrefKind, trace: Trace) -> Result<String, ExpansionError> {
        let id = id.into();
        if id == "" {
            return Ok("".to_string());
        }
        match self.sticky_state.ids.insert(id.clone().into(), IdInfo {
            definition: trace.clone(),
            file: self.current_output_relative(),
            // preview: self.base_dir().join(format!(r#"previews/{}.html"#, id)),
            kind,
        }) {
            Some(info) if !self.second_iteration => return Err(ExpansionError::DuplicateId(info.definition, trace)),
            _ => return Ok(self.resolve_id_to_url(id, trace)?),
        }
    }

    pub(crate) fn create_preview(&mut self, id: impl Into<String>, content: impl Into<String>) -> Result<(), ExpansionError> {
        let id = id.into();
        let content = content.into();
        let _ = fs_extra::dir::create_all(self.base_dir().join("build/previews/"), false);

        let p = self.base_dir().join(format!(r#"build/previews/{}.html"#, id));
        return std::fs::write(&p, &content).map_err(|e| ExpansionError::OutputIO(e, p.clone(), Trace(None)));
    }

    pub(crate) fn create_box_previews(&mut self, content: impl Into<String>) -> Result<(), ExpansionError> {
        let content = content.into();

        let _ = fs_extra::dir::create_all(self.base_dir().join("build/previews/"), false);

        for id in self.box_previews.iter() {
            let p = self.base_dir().join(format!(r#"build/previews/{}.html"#, id));
            // println!("\nbox {}: {:?}: {:?}", id, p, content);
            return std::fs::write(&p, &content).map_err(|e| ExpansionError::OutputIO(e, p.clone(), Trace(None)));
        }

        self.box_previews.clear();
        return Ok(());
    }

    pub(crate) fn create_boxless_previews(&mut self, content: impl Into<String>) -> Result<(), ExpansionError> {
        let content = content.into();

        let _ = fs_extra::dir::create_all(self.base_dir().join("build/previews/"), false);

        for id in self.boxless_previews.iter() {
            let p = self.base_dir().join(format!(r#"build/previews/{}.html"#, id));
            std::fs::write(&p, &content).map_err(|e| ExpansionError::OutputIO(e, p.clone(), Trace(None)))?;
        }

        self.boxless_previews.clear();
        return Ok(());
    }

    pub(crate) fn resolve_id_to_url(&self, id: impl Into<String>, trace: Trace) -> Result<String, ExpansionError> {
        if self.second_iteration {
            let id = id.into();
            match self.sticky_state.ids.get(&id) {
                None => return Err(ExpansionError::UnknownId(trace)),
                Some(info) => {
                    return Ok(format!(
                        "{}{}#{}",
                        self.domain,
                        info.file.to_string_lossy(),
                        id,
                    ));
                }
            }
        } else {
            return Ok("set in second iteration".to_string());
        }
    }

    pub(crate) fn id_to_preview_url(&self, id: impl Into<String>) -> String {
        return format!(
            r###"{}previews/{}.html"###,
            self.domain,
            id.into(),
        );
    }

    pub(crate) fn resolve_defined_to_preview_url(&self, id: impl Into<String>, trace: Trace) -> Result<String, ExpansionError> {
        if self.second_iteration {
            let id = id.into();
            match self.sticky_state.defined.get(&id) {
                Some(info) => {
                    return Ok(info.preview.clone());
                }
                None => {
                    return Err(ExpansionError::UnknownId(trace));
                }
            }
        } else {
            return Ok("set in second iteration".to_string());
        }
    }

    pub(crate) fn register_define(&mut self, defined: impl Into<String>, href: String, preview: String, singular: String, plural: String, trace: Trace) -> Result<(), ExpansionError> {
        let defined = defined.into();
        if defined == "" {
            return Err(ExpansionError::EmptyDefine(trace));
        }

        match self.sticky_state.defined.insert(defined.into(), DefinedInfo {
            definition: trace.clone(),
            href,
            preview,
            singular,
            plural,
        }) {
            Some(info) if !self.second_iteration => return Err(ExpansionError::DuplicateDefine(info.definition, trace)),
            _ => return Ok(()),
        }
    }

    pub(crate) fn resolve_defined_to_url(&self, defined: impl Into<String>, trace: Trace) -> Result<String, ExpansionError> {
        if self.second_iteration {
            let defined = defined.into();
            match self.sticky_state.defined.get(&defined) {
                None => return Err(ExpansionError::UnknownDefine(trace)),
                Some(info) => {
                    return Ok(info.href.clone());
                }
            }
        } else {
            return Ok("set in second iteration".to_string());
        }
    }
}

#[derive(Clone)]
pub(crate) enum BoxKind {
    Exercise,
    Other(OtherBoxKind),
}

impl BoxKind {
    pub(crate) fn class(&self) -> String {
        match self {
            BoxKind::Exercise => "exercise".to_string(),
            BoxKind::Other(OtherBoxKind::Fact) => "fact".to_string(),
            BoxKind::Other(OtherBoxKind::Example) => "example".to_string(),
            BoxKind::Other(OtherBoxKind::Definition) => "definition".to_string(),
        }
    }

    pub(crate) fn exercise() -> Self {
        BoxKind::Exercise
    }

    pub(crate) fn fact() -> Self {
        BoxKind::Other(OtherBoxKind::Fact)
    }

    pub(crate) fn example() -> Self {
        BoxKind::Other(OtherBoxKind::Example)
    }

    pub(crate) fn definition() -> Self {
        BoxKind::Other(OtherBoxKind::Definition)
    }
}

#[derive(Clone)]
pub(crate) enum OtherBoxKind {
    Fact,
    Example,
    Definition,
}

#[derive(Clone)]
pub(crate) struct StickyState {
    pub ids: HashMap<String, IdInfo>,
    pub hsections: HashMap<String, HSectionInfo>,
    pub boxes: HashMap<String, BoxInfo>,
    pub defined: HashMap<String, DefinedInfo>,
}

impl StickyState {
    fn new() -> Self {
        StickyState {
            ids: HashMap::new(),
            hsections: HashMap::new(),
            boxes: HashMap::new(),
            defined: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) enum CrefKind {
    HSection,
    Box,
    BoxlessDefinition,
}

#[derive(Clone)]
pub(crate) struct IdInfo {
    pub definition: Trace,
    pub file: PathBuf,
    pub kind: CrefKind,
}

#[derive(Clone)]
pub struct HSectionInfo {
    pub name: String, // "Chapter", "Section", etc.
    pub numbering: String,
}

#[derive(Clone)]
pub(crate) struct BoxInfo {
    pub name: String, // "Theorem", "Lemma", etc.
    pub numbering: String,
    pub kind: BoxKind,
}

#[derive(Clone)]
pub(crate) struct DefinedInfo {
    pub definition: Trace,
    pub href: String,
    pub preview: String,
    pub singular: String,
    pub plural: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let c = RunConfiguration {
        entrypoint: args[1].clone().into(),
    };
    run(c);
}
