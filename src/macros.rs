use sourcefile::SourceFile;
use std::io;
use std::fs;
use std::collections::BTreeMap;
use std::path::PathBuf;

use ropey::Rope;
use thiserror::Error;
use serde::Deserialize;

use crate::{Yatt, print_trace, CrefKind, BoxKind};
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
    Parse(#[from] parse::ParseError),
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
    EmptyDefine(Trace),
    #[error("never printed")]
    UnknownDefine(Trace),
    #[error("never printed")]
    DuplicateDefine(Trace /* definition */, Trace /* redefinition */),
    #[error("never printed")]
    UnknownId(Trace),
    #[error("never printed")]
    DuplicateId(Trace /* definition */, Trace /* redefinition */),
    #[error("never printed")]
    UnknownMathId(Trace, String),
    #[error("never printed")]
    DuplicateMathId(Trace /* redefinition */, String /* id */),
    #[error("never printed")]
    HSectionTooManyLevels(Trace),
    #[error("never printed")]
    CrefBoxlessDefinition(Trace),
    #[cfg(unix)]
    #[error("never printed")]
    TeX(katex::Error, Trace),
}

impl ExpansionError {
    pub fn print_expansion_error(&self, source: &SourceFile) {
        match self {
            ExpansionError::Parse(e) => {
                e.print_parse_error(source);
            }
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
            ExpansionError::HSectionTooManyLevels(t) => {
                println!("Cannot nest the `hsection` macro more than five times.");
                print_trace(t.clone(), source, false);
            }
            ExpansionError::CrefBoxlessDefinition(t) => {
                println!("Cannot use `§cref` to reference a boxless definition");
                print_trace(t.clone(), source, false);
            }
            ExpansionError::DuplicateId(definition, redefinition) => {
                println!("Cannot define the same id multiple times.");
                println!("First definition:");
                print_trace(definition.clone(), source, true);
                println!("Second definition:");
                print_trace(redefinition.clone(), source, true);
            }
            ExpansionError::UnknownId(id) => {
                println!("Tried to reference undefined id.");
                print_trace(id.clone(), source, true);
            }
            ExpansionError::UnknownMathId(t, id) => {
                println!("Must set the id for each math macro that links to a definition.");
                println!("Macro: {}", id);
                println!("At:");
                print_trace(t.clone(), source, true);
            }
            ExpansionError::DuplicateMathId(redefinition, id) => {
                println!("Cannot define the id corresponding to a math macro multiple times.");
                println!("Id: {}", id);
                println!("Redefinition at:");
                print_trace(redefinition.clone(), source, true);
            }
            ExpansionError::DuplicateDefine(definition, redefinition) => {
                println!("Cannot define the same name multiple times.");
                println!("First definition:");
                print_trace(definition.clone(), source, true);
                println!("Second definition:");
                print_trace(redefinition.clone(), source, true);
            }
            ExpansionError::UnknownDefine(id) => {
                println!("Tried to reference undefined define.");
                print_trace(id.clone(), source, true);
            }
            ExpansionError::EmptyDefine(id) => {
                println!("Cannot define the empty name.");
                print_trace(id.clone(), source, true);
            }
            #[cfg(unix)]
            ExpansionError::TeX(e, t) => {
                println!("Invalid tex input.\n");
                println!("{:?}\n", e);
                print_trace(t.clone(), source, false);
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
    TeX(TeX, Vec<Out>, bool),
    HtmlTag(String, TagParams, Vec<Out>),
    Input([PathBuf; 1], Vec<Out>),
    Output([PathBuf; 1], Vec<Out>, bool),
    Cref(Cref, Vec<Out>),
}

#[derive(Clone)]
pub(crate) enum OutInternal {
    Many(Vec<OutInternal>),
    Text(Rope, Trace),
    EmptyMacro(Trace, (), Vec<OutInternal>),
    Const(Trace, (), Vec<OutInternal>, &'static str),
    HtmlTag(Trace, String, TagParams, Vec<OutInternal>),
    P(Trace, (), Vec<OutInternal>),
    Input(Trace, [PathBuf; 1], Vec<OutInternal>),
    Output(Trace, [PathBuf; 1], Vec<OutInternal>, bool),
    CopyAll(Trace, [PathBuf; 2], Vec<OutInternal>),
    Template(Trace, Template, Vec<OutInternal>),
    HSection(Trace, HSection, Vec<OutInternal>),
    Box(Trace, BoxParams, Vec<OutInternal>, BoxKind, String),
    Fact(Trace, BoxParams, Vec<OutInternal>, String),
    Proof(Trace, Proof, Vec<OutInternal>),
    Define(Trace, Define, Vec<OutInternal>, bool /* is there custom definition text */),
    Cref(Trace, Cref, Vec<OutInternal>),
    TeX(Trace, TeX, Vec<OutInternal>, bool),
    Cwd(Trace, (), Vec<OutInternal>),
    SetDomain(Trace, String, Vec<OutInternal>),
    ReferenceDefined(Trace, (), Vec<OutInternal>, bool /*capitalize*/, bool /*plural*/, bool /*fake define*/),
    SetMathId(Trace, SetMathId, Vec<OutInternal>),
    MathMacro(Trace, (), Vec<OutInternal>, String /* id */, String /* tex */),
    MathSet(Trace, (), Vec<OutInternal>),
    MathEnv(Trace, (), Vec<OutInternal>, &'static str /*environment name*/),
    Link(Trace, (), Vec<OutInternal>),
    Captioned(Trace, (), Vec<OutInternal>),
    Enclose(Trace, (), Vec<OutInternal>, &'static str, &'static str),
    EncloseMath(Trace, (), Vec<OutInternal>, &'static str /* id */, &'static str, &'static str),
}

impl OutInternal {
    fn trace(&self) -> Trace {
        match self {
            OutInternal::Text(_, t)
            | OutInternal::EmptyMacro(t, _, _)
            | OutInternal::HtmlTag(t, _, _, _)
            | OutInternal::Input(t, _, _)
            | OutInternal::Output(t, _, _, _)
            | OutInternal::CopyAll(t, _, _)
            | OutInternal::Template(t, _, _)
            | OutInternal::HSection(t, _, _) => t.clone(),
            _ => unimplemented!(),
        }
    }
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
            arguments_exact(0, &args, &trace)?;
            return Ok(Rope::new());
        }

        OutInternal::Const(trace, _params, args, text) => {
            arguments_exact(0, &args, &trace)?;
            return Ok(text.into());
        }

        OutInternal::Cwd(trace, _params, args) => {
            arguments_exact(0, &args, &trace)?;
            return Ok(y.state.cwd().to_string_lossy().into());
        }

        OutInternal::HtmlTag(trace, tag, params, args) => {
            return down_macro(|p, n, y, trace| {
                html_tag(&tag, p, n, y, trace)
            }, &params, args, trace, y);
        }

        OutInternal::Input(span, path, args) => {
            arguments_exact(0, &args, &span)?;
            let path = &path[0];

            let old_current_file = y.state.current_file.clone();
            if path.is_absolute() {
                y.state.current_file = y.state.base_dir().join(path.strip_prefix("/").unwrap());
            } else {
                y.state.current_file = y.state.base_dir().join(path);
            }

            match fs::read_to_string(&y.state.current_file) {
                Err(e) => return Err(ExpansionError::InputIO(e, y.state.current_file.clone(), span)),
                Ok(entry) => {
                    let source_offset = y.source.contents.len();
                    y.source.add_file_raw(&y.state.entrypoint.to_string_lossy(), &entry);
                    let ast = parse::parse(&entry, y, source_offset)?;
                    let r = expand(ast, y)?;
                    y.state.current_file = old_current_file;
                    return Ok(r);
                }
            }
        }

        OutInternal::Output(span, path, args, tee) => {
            arguments_exact(1, &args, &span)?;

            let old_output = y.state.current_output.clone();

            let path = &path[0];

            let p = if path.is_absolute() {
                y.state.base_dir().join("build/").join(&path.strip_prefix("/").unwrap())
            } else {
                y.state.cwd().join("build/").join(path)
            };

            let mut dirname = p.clone();
            dirname.pop();
            let _ = fs_extra::dir::create_all(dirname, false);

            y.state.current_output = p.clone();

            let r = up_macro(|_path, args, y, span| {
                let content = args[0].to_string();

                if y.state.second_iteration {
                    match fs::write(&p, &content) {
                        Err(e) => return Err(ExpansionError::OutputIO(e, p.clone(), span)),
                        Ok(()) => {
                            if tee {
                                return Ok(args[0].clone());
                            } else {
                                return Ok(Rope::new());
                            }
                        }
                    }
                } else {
                    if tee {
                        return Ok(args[0].clone());
                    } else {
                        return Ok(Rope::new());
                    }
                }
            }, &path, args, span, y);

            y.state.current_output = old_output;

            return r;
        }

        OutInternal::CopyAll(span, params, args) => {
            arguments_exact(0, &args, &span)?;

            if y.state.second_iteration {
                let from = &params[0];
                let to = PathBuf::from("build/").join(&params[1]);

                let mut opts = fs_extra::dir::CopyOptions::new();
                opts.copy_inside = true;
                opts.overwrite = true;

                fs_extra::copy_items(&params[0..1], &to, &opts)
                .map_err(|e| ExpansionError::CopyAll(e, from.clone(), to.clone(), span))?;
            }

            return Ok(Rope::new());
        }

        OutInternal::Template(trace, params, args) => {
            arguments_exact(2, &args, &trace)?;
            return down_macro(|_p, _n, _y, _trace| {
                let outs = vec![
                    Out::Text(r###"<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet" href="./assets/katex.min.css">
        <link rel="stylesheet" href="./assets/fonts.css">
        <link rel="stylesheet" href="./assets/main.css">
        <script src="./assets/floating-ui.core.min.js"></script>
        <script src="./assets/floating-ui.dom.min.js"></script>"###.into()),
                    Out::Argument(0),
                    Out::Text(r###"
    </head>
    <body>
"###.into()),
                    Out::Argument(1),
                    Out::Text(r###"
        <script src="./assets/previews.js"></script>
    </body>
</html>
"###.into()),
                ];


                return Ok(Out::Many(outs));
            }, &params, args, trace, y);
        }

        OutInternal::TeX(span, path, args, display) => {
            arguments_exact(1, &args, &span)?;

            return up_macro(|_, args, _y, span| {
                #[cfg(unix)]
                {
                    let content = args[0].to_string();
                    let opts = katex::Opts::builder()
                        .display_mode(display)
                        .throw_on_error(true)
                        .trust(true)
                        .build().unwrap();

                    return Ok(katex::render_with_opts(&content, &opts).map_err(|e| ExpansionError::TeX(e, span.clone()))?.into());
                }

                #[cfg(not(unix))]
                {
                    return Ok("cannot render KaTeX on non-unix system".into());
                }
            }, &path, args, span, y);
        }

        OutInternal::SetDomain(span, path, args) => {
            arguments_exact(1, &args, &span)?;

            return up_macro(|_, args, y, _span| {
                y.state.domain = args[0].to_string();
                return Ok("".into());
            }, &path, args, span, y);
        }

        OutInternal::HSection(trace, params, args) => {
            arguments_exact(3, &args, &trace)?;

            y.state.hsection_level += 1;
            if y.state.hsection_level > 5 {
                return Err(ExpansionError::HSectionTooManyLevels(trace));
            }

            let level = y.state.hsection_level;

            y.state.hsection_current_count[level] += 1;

            if y.state.box_exercise_level == level {
                y.state.box_exercise_current_count = 0;
            }
            if y.state.box_other_level == level {
                y.state.box_other_current_count = 0;
            }

            let mut numbering = "".to_string();
            for i in 0..6 {
                if y.state.hsection_current_count[i] != 0 {
                    numbering.push_str(&format!("{}", y.state.hsection_current_count[i]));

                    if i < 5 && y.state.hsection_current_count[i + 1] != 0 {
                        numbering.push('.');
                    }
                }
            }

            let id_trace = args[0].trace();
            let r = up_macro(|_p, args, y, _trace| {
                let url = y.state.register_id(&args[0], CrefKind::HSection, id_trace.clone())?;
                y.state.sticky_state.hsections.insert(args[0].to_string(), crate::HSectionInfo {
                    name: y.state.hsection_name[level].clone(),
                    numbering: numbering.clone(),
                });

                return Ok(format!(r###"<section>
    <h{} id="{}"><a href="{}">{}{}{}{}</a></h{}>
    {}
</section>"###,
                    level + 1,
                    args[0],
                    url,
                    if y.state.hsection_render_number[level] { &y.state.hsection_pre_number[level] } else { "" },
                    if y.state.hsection_render_number[level] { &numbering } else { "" },
                    if y.state.hsection_render_number[level] { &y.state.hsection_post_number[level] } else { "" },
                    args[1],
                    level + 1,
                    args[2]).into(),
                );
            }, &params, args, trace, y);

            for i in 0..6 {
                if i > level {
                    y.state.hsection_current_count[i] = 0;
                }
            }
            y.state.hsection_level -= 1;

            return r;
        }

        OutInternal::Box(trace, params, args, kind, name) => {
            arguments_gte(1, &args, &trace)?;
            arguments_lt(4, &args, &trace)?;

            let (hsection_level, number) = match kind {
                BoxKind::Exercise => {
                    y.state.box_exercise_current_count += 1;
                    (y.state.box_exercise_level, y.state.box_exercise_current_count)
                }
                BoxKind::Other(_) => {
                    y.state.box_other_current_count += 1;
                    (y.state.box_other_level, y.state.box_other_current_count)
                }
                BoxKind::Proof => (0, 0), // not used, dummy values
            };

            let mut numbering = "".to_string();
            for i in 0..=hsection_level {
                if y.state.hsection_current_count[i] != 0 {
                    numbering.push_str(&format!("{}", y.state.hsection_current_count[i]));
                    numbering.push('.');
                }
            }
            numbering.push_str(&format!("{}", number));

            let id = params.0[0].clone();
            y.state.box_current = Some(id.to_string());

            let id_trace = Trace(None);
            let r = up_macro(|_p, args, y, _trace| {
                let url = y.state.register_id(&id.clone(), CrefKind::Box, id_trace.clone())?;
                y.state.sticky_state.boxes.insert(id.to_string(), crate::BoxInfo {
                    name: name.clone(),
                    numbering: numbering.clone(),
                    kind: kind.clone(),
                });



                let box_html = format!(r###"<article class="{}" id="{}">
    <h6><a href="{}">{} {}{}{}</a></h6>{}    {}
</article>"###,
                    kind.class(),
                    id,
                    url,
                    name,
                    numbering,
                    if args.len() == 3 { ": " } else { "" },
                    if args.len() == 3 { args[0].clone() } else { Rope::new() },
                    if args.len() >= 2 {
                        format!(r###"
                        <div class="assumptions">
                            {}
                        </div>
"###, args[args.len() - 2])
                    } else {
                        "".to_string()
                    },
                    args[args.len() - 1],
                );

                y.state.create_preview(&id, &box_html)?;
                y.state.create_box_previews(&box_html)?;
                return Ok(box_html.into());
            }, &params, args, trace, y);

            y.state.box_current = None;

            return r;
        }

        OutInternal::Fact(trace, params, args, name) => {
            arguments_gte(2, &args, &trace)?;
            arguments_lt(5, &args, &trace)?;

            y.state.box_other_current_count += 1;
            let hsection_level = y.state.box_other_level;
            let number = y.state.box_other_current_count;

            let mut numbering = "".to_string();
            for i in 0..=hsection_level {
                if y.state.hsection_current_count[i] != 0 {
                    numbering.push_str(&format!("{}", y.state.hsection_current_count[i]));
                    numbering.push('.');
                }
            }
            numbering.push_str(&format!("{}", number));

            let id = params.0[0].clone();
            y.state.box_current = Some(id.to_string());

            let id_trace = Trace(None);
            let r = up_macro(|_p, args, y, _trace| {
                let url = y.state.register_id(&id.clone(), CrefKind::Box, id_trace.clone())?;
                y.state.sticky_state.boxes.insert(id.to_string(), crate::BoxInfo {
                    name: name.clone(),
                    numbering: numbering.clone(),
                    kind: BoxKind::fact(),
                });



                let box_html = format!(r###"<article class="{}" id="{}">
    <h6><a href="{}">{} {}{}{}</a></h6>{}
    <div class="claim">{}</div>
    <div class="proof_body">{}</div>
</article>"###,
                    BoxKind::fact().class(),
                    id,
                    url,
                    name,
                    numbering,
                    if args.len() == 4 { ": " } else { "" },
                    if args.len() == 4 { args[0].clone() } else { Rope::new() },
                    if args.len() >= 3 {
                        format!(r###"
                        <div class="assumptions">
                            {}
                        </div>
"###, args[args.len() - 3])
                    } else {
                        "".to_string()
                    },
                    args[args.len() - 2],
                    args[args.len() - 1],
                );

                y.state.create_preview(&id, &box_html)?;
                y.state.create_box_previews(&box_html)?;
                return Ok(box_html.into());
            }, &params, args, trace, y);

            y.state.box_current = None;

            return r;
        }

        OutInternal::Proof(trace, params, args) => {
            arguments_gte(2, &args, &trace)?;
            arguments_lt(4, &args, &trace)?;

            let kind = BoxKind::proof();
            let name = "Proof";

            let id = format!("proof_{}", params.0[0]);
            y.state.box_current = Some(id.to_string());

            let id_trace = Trace(None);
            let r = up_macro(|_p, args, y, _trace| {
                let url = y.state.register_id(&id.clone(), CrefKind::Box, id_trace.clone())?;
                y.state.sticky_state.boxes.insert(id.to_string(), crate::BoxInfo {
                    name: name.to_string(),
                    numbering: "".to_string(),
                    kind: kind.clone(),
                });
                let claim_name = y.state.claim_name(&params.0[0], id_trace.clone())?;

                let box_html = format!(r###"<article class="{}" id="{}">
    <h6><a href="{}">Proof of {}</a></h6>{}
    <div class="claim">{}</div>
    <div class="proof_body">{}</div>
</article>"###,
                    kind.class(),
                    id,
                    url,
                    claim_name,
                    if args.len() >= 2 {
                        format!(r###"
                        <div class="assumptions">
                            {}
                        </div>
"###, args[0])
                    } else {
                        "".to_string()
                    },
                    &args[args.len() - 2],
                    &args[args.len() - 1],
                );

                y.state.create_preview(&id, &box_html)?;
                y.state.create_box_previews(&box_html)?;
                return Ok(box_html.into());
            }, &params, args, trace, y);

            y.state.box_current = None;

            return r;
        }

        OutInternal::P(trace, params, args) => {
            arguments_exact(1, &args, &trace)?;

            let r = up_macro(|_p, args, y, _trace| {
                let p_html = format!(r###"<p>{}</p>"###, args[0]);
                y.state.create_boxless_previews(&p_html)?;
                return Ok(p_html.into());
            }, &params, args, trace, y);

            return r;
        }

        OutInternal::Cref(trace, params, args) => {
            arguments_gte(1, &args, &trace)?;
            arguments_lt(3, &args, &trace)?;

            let id_trace = args[0].trace();

            if y.state.second_iteration {
                return up_macro(|_p, args, y, _trace| {
                    let id = &args[0];
                    match y.state.sticky_state.ids.get(&id.to_string()) {
                        None => return Err(ExpansionError::UnknownId(id_trace.clone())),
                        Some(info) => {
                            let url = y.state.resolve_id_to_url(id, id_trace.clone())?;

                            match info.kind {
                                CrefKind::HSection => {
                                    let hsection_info = y.state.sticky_state.hsections.get(&id.to_string()).unwrap();
                                    let label = if args.len() == 2 {
                                        args[1].to_string()
                                    } else {
                                        format!("{}&nbsp;{}", hsection_info.name, hsection_info.numbering)
                                    };
                                    let tag = format!(
                                        r###"<a class="ref" href="{}">{}</a>"###,
                                        url,
                                        label,
                                    );
                                    return Ok(tag.into());
                                }

                                CrefKind::Box => {
                                    let box_info = y.state.sticky_state.boxes.get(&id.to_string()).unwrap();
                                    match box_info.kind {
                                        BoxKind::Proof => {
                                            let claim_name = y.state.claim_name(&id.to_string(), id_trace.clone())?;
                                            let tag = format!(
                                                r###"<a class="ref {}" href="{}" data-preview="{}">proof of {}</a>"###,
                                                box_info.kind.class(),
                                                url,
                                                y.state.id_to_preview_url(id),
                                                claim_name,
                                            );
                                            return Ok(tag.into());
                                        }
                                        _ => {
                                            let tag = format!(
                                                r###"<a class="ref {}" href="{}" data-preview="{}">{}&nbsp;{}</a>"###,
                                                box_info.kind.class(),
                                                url,
                                                y.state.id_to_preview_url(id),
                                                box_info.name,
                                                box_info.numbering,
                                            );
                                            return Ok(tag.into());
                                        }
                                    }
                                }

                                CrefKind::BoxlessDefinition => return Err(ExpansionError::CrefBoxlessDefinition(id_trace.clone())),
                            }
                        }
                    }
                }, &params, args, trace, y);
            } else {
                return Ok(Rope::new());
            }
        }

        OutInternal::Define(trace, params, args, custom_text) => {
            arguments_gte(if custom_text {2} else {1}, &args, &trace)?;
            arguments_lt(if custom_text {5} else {4}, &args, &trace)?;

            return up_macro(|_p, args, y, trace| {
                let (target_id, boxless) = match &y.state.box_current {
                    None => (params.0[0].clone(), true),
                    Some(id) => (id.to_string(), false),
                };

                if custom_text {
                    y.state.register_id(&target_id.clone(), CrefKind::BoxlessDefinition, Trace(None))?;

                    let p = y.state.base_dir().join(format!(r#"build/previews/{}.html"#, target_id));
                    let _ = std::fs::write(&p, format!(r###"<article>{}</article>"###, &args[args.len() - 1])).map_err(|e| ExpansionError::OutputIO(e, p.clone(), Trace(None)))?;
                } else {
                    if boxless {
                        y.state.register_id(&target_id.clone(), CrefKind::BoxlessDefinition, Trace(None))?;
                        y.state.boxless_previews.insert(target_id.to_string());
                    } else {
                        y.state.box_previews.insert(target_id.to_string());
                    }
                }

                let defined = args[0].clone();
                let href = if boxless {
                    format!(
                        "{}{}#{}",
                        y.state.domain,
                        y.state.current_output_relative().to_string_lossy(),
                        target_id,
                    )
                } else {
                    y.state.resolve_id_to_url(&target_id, Trace(None))?
                };
                let preview_url = y.state.id_to_preview_url(target_id.clone());
                let singular = args[0].to_string();
                let plural = if args.len() >= (if custom_text {3} else {2}) {
                    args[1].to_string()
                } else {
                    format!("{}s", singular)
                };

                y.state.register_define(defined, href.clone(), preview_url, singular, plural, trace)?;

                return Ok(format!(r###"<dfn{}><a href="{}">{}</a></dfn>"###,
                    if boxless { format!(r#" id="{}""#, target_id) } else { "".to_string() },
                    href,
                    if args.len() >= (if custom_text {4} else {3}) { args[2].clone() } else { args[0].clone() },
                ).into());
            }, &params, args, trace, y);
        }

        OutInternal::ReferenceDefined(trace, params, args, capitalize, pluralize, fakedef) => {
            arguments_gte(1, &args, &trace)?;
            arguments_lt(3, &args, &trace)?;

            let id_trace = args[0].trace();

            if y.state.second_iteration {
                return up_macro(|_p, args, y, _trace| {
                    let id = args[0].to_string();

                    match y.state.sticky_state.defined.get(&id) {
                        None => return Err(ExpansionError::UnknownDefine(id_trace.clone())),
                        Some(info) => {
                            let name = if args.len() == 2 {
                                args[1].to_string()
                            } else {
                                let tmp = if pluralize {
                                    &info.plural
                                } else {
                                    &info.singular
                                };
                                if capitalize {
                                    some_kind_of_uppercase_first_letter(tmp)
                                } else {
                                    tmp.to_string()
                                }
                            };

                            if fakedef {
                                return Ok(format!(
                                    r###"<dfn id="{}"><a href="{}">{}</a></dfn>"###,
                                    &id,
                                    y.state.resolve_defined_to_preview_url(&id, id_trace.clone())?,
                                    name,
                                ).into());
                            } else {
                                return Ok(format!(
                                    r###"<a class="ref definition" href="{}" data-preview="{}">{}</a>"###,
                                    info.href,
                                    y.state.resolve_defined_to_preview_url(id, id_trace.clone())?,
                                    name,
                                ).into());
                            }
                        }
                    }
                }, &params, args, trace, y);
            } else {
                return Ok(Rope::new());
            }
        }

        OutInternal::Enclose(trace, params, args, pre, post) => {
            arguments_exact(1, &args, &trace)?;

            return down_macro(|_p, _n, _y, _trace| {
                return Ok(Out::Many(vec![
                        Out::Text(pre.into()),
                        Out::Argument(0),
                        Out::Text(post.into()),
                    ]));
            }, &params, args, trace, y);
        }

        OutInternal::Link(trace, params, args) => {
            arguments_exact(2, &args, &trace)?;

            return down_macro(|_p, _n, _y, _trace| {
                return Ok(Out::Many(vec![
                        Out::Text(r###"<a class="outlink" href=""###.into()),
                        Out::Argument(1),
                        Out::Text(r###"">"###.into()),
                        Out::Argument(0),
                        Out::Text(r###"</a>"###.into()),
                    ]));
            }, &params, args, trace, y);
        }

        OutInternal::Captioned(trace, params, args) => {
            arguments_exact(2, &args, &trace)?;

            return down_macro(|_p, _n, _y, _trace| {
                return Ok(Out::Many(vec![
                        Out::Text(r###"<div class="captioned">"###.into()),
                        Out::Argument(0),
                        Out::Text(r###"<div class="caption">"###.into()),
                        Out::Argument(1),
                        Out::Text(r###"</div></div>"###.into()),
                    ]));
            }, &params, args, trace, y);
        }

        OutInternal::SetMathId(trace, params, args) => {
            arguments_exact(0, &args, &trace)?;

            if y.state.second_iteration {
                return Ok(Rope::new());
            } else {
                match y.state.sticky_state.math_definitions.insert(params.0[0].clone(), params.0[1].to_string()) {
                    None => return Ok(Rope::new()),
                    Some(_) => return Err(ExpansionError::DuplicateMathId(trace.clone(), params.0[0].to_string())),
                }
            }
        }

        OutInternal::MathMacro(trace, _params, args, math_id, tex) => {
            arguments_exact(0, &args, &trace)?;

            if y.state.second_iteration {
                match y.state.sticky_state.math_definitions.get(&math_id) {
                    None => return Err(ExpansionError::UnknownMathId(trace, math_id.to_string())),
                    Some(id) => {
                        let url = y.state.resolve_id_to_url(id, trace)?;
                        let preview_url = y.state.id_to_preview_url(id);
                        return Ok(format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{{}}}}}"###, preview_url, url, tex).into());
                    }
                }
            } else {
                return Ok(Rope::new());
            }
        }

        OutInternal::EncloseMath(trace, params, args, math_id, pre, post) => {
            arguments_exact(1, &args, &trace)?;

            return down_macro(|_p, _n, y, trace| {
                if y.state.second_iteration {
                    match y.state.sticky_state.math_definitions.get(math_id) {
                        None => return Err(ExpansionError::UnknownMathId(trace, math_id.to_string())),
                        Some(id) => {
                            let url = y.state.resolve_id_to_url(id, trace)?;
                            let preview_url = y.state.id_to_preview_url(id);

                            let pre_tex = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{{}}}}}"###, preview_url, url, pre);
                            let post_tex = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{{}}}}}"###, preview_url, url, post);

                            return Ok(Out::Many(vec![
                                    Out::Text(pre_tex.into()),
                                    Out::Argument(0),
                                    Out::Text(post_tex.into()),
                                ]));
                        }
                    }
                } else {
                    return Ok(Out::Argument(0));
                }
            }, &params, args, trace, y);
        }

        OutInternal::MathSet(trace, params, args) => {
            return down_macro(|_p, n, y, trace| {
                if y.state.second_iteration {
                    match y.state.sticky_state.math_definitions.get("set") {
                        None => return Err(ExpansionError::UnknownMathId(trace, "set".to_string())),
                        Some(id) => {
                            let url = y.state.resolve_id_to_url(id, trace)?;
                            let preview_url = y.state.id_to_preview_url(id);
                            if n == 0 {
                                return Ok(Out::Text(format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{\emptyset}}}}"###, preview_url, url).into()));
                            } else {
                                // KaTeX doesn't like how \left and \right are used within other stuff?
                                // let left_delimiter = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{\left\lbrace}}}}"###, preview_url, url);
                                // let right_delimiter = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{\right\rbrace}}}}"###, preview_url, url);
                                let left_delimiter = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{\lbrace}}}}"###, preview_url, url);
                                let right_delimiter = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{\rbrace}}}}"###, preview_url, url);

                                let mut outs = vec![Out::Text(left_delimiter.into())];
                                for i in 0..n {
                                    if i != 0 {
                                        outs.push(Out::Text(", ".into()));
                                    }
                                    outs.push(Out::Argument(i));
                                }
                                outs.push(Out::Text(right_delimiter.into()));

                                return Ok(Out::Many(outs));
                            }
                        }
                    }
                } else {
                    let mut outs = vec![];
                    for i in 0..n {
                        outs.push(Out::Argument(i));
                    }
                    return Ok(Out::Many(outs));
                }
            }, &params, args, trace, y);
        }

        OutInternal::MathEnv(trace, params, args, env) => {
            return down_macro(|_p, n, _y, _trace| {
                let mut outs = vec![Out::Text(format!(r###"\begin{{{}}}"###, env).into())];
                for i in 0..n {
                    if i != 0 {
                        outs.push(Out::Text(r###"\\
"###.into()));
                    }
                    outs.push(Out::Argument(i));
                }
                outs.push(Out::Text(format!(r###"\end{{{}}}"###, env).into()));

                return Ok(Out::TeX(TeX::default(), vec![Out::Many(outs)], true));
            }, &params, args, trace, y);
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

fn arguments_lt(n: usize, args: &[OutInternal], span: &Trace) -> Result<(), ExpansionError> {
    if args.len() >= n {
        return Err(ExpansionError::ArgumentNumber(args.len(), span.clone()));
    } else {
        return Ok(());
    }
}

fn arguments_gte(n: usize, args: &[OutInternal], span: &Trace) -> Result<(), ExpansionError> {
    if args.len() < n {
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

        Out::Cref(params, a) => return Ok(OutInternal::Cref(Trace(None), params, outs_to_internals(a, args)?)),

        Out::TeX(path, a, display) => return Ok(OutInternal::TeX(Trace(None), path, outs_to_internals(a, args)?, display)),
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

        let start_middle = r###"" class=""###;
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

// https://stackoverflow.com/a/38406885
fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

#[derive(Deserialize, Clone)]
pub struct Template;

impl Default for Template {
    fn default() -> Self {
        Template
    }
}

#[derive(Deserialize, Clone)]
pub struct HSection;

impl Default for HSection {
    fn default() -> Self {
        HSection
    }
}

#[derive(Deserialize, Clone)]
pub struct Cref;

impl Default for Cref {
    fn default() -> Self {
        Cref
    }
}

#[derive(Deserialize, Clone)]
pub struct BoxParams([String; 1]);

impl Default for BoxParams {
    fn default() -> Self {
        BoxParams(["".to_string()])
    }
}

#[derive(Deserialize, Clone)]
pub struct Proof([String; 1]);

impl Default for Proof {
    fn default() -> Self {
        Proof(["".to_string()])
    }
}

#[derive(Deserialize, Clone)]
pub struct Define([String; 1]);

impl Default for Define {
    fn default() -> Self {
        Define(["".to_string()])
    }
}

#[derive(Deserialize, Clone)]
pub struct TeX;

impl Default for TeX {
    fn default() -> Self {
        TeX
    }
}

#[derive(Deserialize, Clone)]
pub struct SetMathId([String; 2]);

impl Default for SetMathId {
    fn default() -> Self {
        SetMathId(["".to_string(), "".to_string()])
    }
}
