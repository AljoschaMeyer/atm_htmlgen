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
    #[error("never printed")]
    CaseNotInCases(Trace),
    #[cfg(unix)]
    #[error("never printed")]
    TeX(katex::Error, Trace),
    #[error("never printed")]
    AlreadyMathmode(Trace),
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
            ExpansionError::CaseNotInCases(t) => {
                println!("Cannot use `§case` outside of `§cases`");
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
            ExpansionError::AlreadyMathmode(t) => {
                println!("Cannot enter math mode while already in math mode.");
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
    MathFunctionParens(MathSet, Vec<Out>),
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
    HSection(Trace, HSection, Vec<OutInternal>, bool /*no numbering*/),
    ChapterNav(Trace, (), Vec<OutInternal>),
    Box(Trace, BoxParams, Vec<OutInternal>, BoxKind, String),
    Fact(Trace, BoxParams, Vec<OutInternal>, String, bool /*no numbering*/),
    Proof(Trace, Proof, Vec<OutInternal>),
    Toggled(Trace, Toggled, Vec<OutInternal>, &'static str, &'static str),
    Define(Trace, Define, Vec<OutInternal>, bool /* is there custom definition text */),
    Cref(Trace, Cref, Vec<OutInternal>),
    TeX(Trace, TeX, Vec<OutInternal>, bool),
    Fleqn(Trace, (), Vec<OutInternal>),
    Cwd(Trace, (), Vec<OutInternal>),
    SetDomain(Trace, String, Vec<OutInternal>),
    ReferenceDefined(Trace, (), Vec<OutInternal>, bool /*capitalize*/, bool /*plural*/, bool /*fake define*/),
    SetMathId(Trace, SetMathId, Vec<OutInternal>),
    MathMacro(Trace, MathMacro, Vec<OutInternal>, String /* id */, String /* tex */),
    MathSet(Trace, MathSet, Vec<OutInternal>),
    MathGroupingParens(Trace, MathSet, Vec<OutInternal>),
    MathFunctionParens(Trace, MathSet, Vec<OutInternal>),
    MathSetBuilder(Trace, MathSetBuilder, Vec<OutInternal>),
    MathEnv(Trace, (), Vec<OutInternal>, &'static str /*environment name*/),
    Link(Trace, (), Vec<OutInternal>),
    Captioned(Trace, (), Vec<OutInternal>),
    Enclose(Trace, (), Vec<OutInternal>, &'static str, &'static str),
    Enclose2(Trace, (), Vec<OutInternal>, &'static str, &'static str, &'static str),
    EncloseMath(Trace, (), Vec<OutInternal>, &'static str /* id */, &'static str, &'static str),
    EncloseFunctionApplication(Trace, MathSet, Vec<OutInternal>, String /* id */, &'static str /* id */),
    Cases(Trace, (), Vec<OutInternal>),
    Case(Trace, Case, Vec<OutInternal>),
    Drop(Trace, (), Vec<OutInternal>),
    ProofPart(Trace, (), Vec<OutInternal>),
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
            | OutInternal::HSection(t, _, _, _) => t.clone(),
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
        <meta charset="UTF-8">
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
            arguments_gte(1, &args, &span)?;
            arguments_lt(4, &args, &span)?;

            y.state.enable_mathmode(&span)?;

            return up_macro(|_, args, y, span| {
                y.state.disable_mathmode(&span)?;

                #[cfg(unix)]
                {
                    let pre = if args.len() == 3 {
                        format!(r###"\text{{{}}}"###, args[0].to_string())
                    } else {
                        "".to_string()
                    };
                    let post = if args.len() > 1 {
                        format!(r###"\text{{{}}}"###, args[args.len() - 1].to_string())
                    } else {
                        "".to_string()
                    };
                    let content = format!(
                        r###"{}{}{}"###,
                        pre,
                        if args.len() == 1 {args[0].to_string()} else {args[args.len() - 2].to_string()},
                        post,
                    );
                    let opts = katex::Opts::builder()
                        .display_mode(display)
                        .throw_on_error(true)
                        .trust(true)
                        .fleqn(y.state.fleqn)
                        .build().unwrap();

                    return Ok(katex::render_with_opts(&content, &opts).map_err(|e| ExpansionError::TeX(e, span.clone()))?.into());
                }

                #[cfg(not(unix))]
                {
                    return Ok("cannot render KaTeX on non-unix system".into());
                }
            }, &path, args, span, y);
        }

        OutInternal::Fleqn(span, path, args) => {
            arguments_exact(1, &args, &span)?;

            y.state.fleqn = !y.state.fleqn;

            return up_macro(|_, args, y, _span| {
                y.state.fleqn = !y.state.fleqn;
                return Ok(args[0].clone());
            }, &path, args, span, y);
        }

        OutInternal::SetDomain(span, path, args) => {
            arguments_exact(1, &args, &span)?;

            return up_macro(|_, args, y, _span| {
                y.state.domain = args[0].to_string();
                return Ok("".into());
            }, &path, args, span, y);
        }

        OutInternal::HSection(trace, params, args, no_numbering) => {
            arguments_exact(2, &args, &trace)?;

            y.state.hsection_level += 1;
            if y.state.hsection_level > 5 {
                return Err(ExpansionError::HSectionTooManyLevels(trace));
            }

            let level = y.state.hsection_level;

            if !no_numbering {
                y.state.hsection_current_count[level] += 1;
            }

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
            if no_numbering {
                numbering = "".to_string();
            }

            let id_trace = args[0].trace();

            y.state.sticky_state.hsections_structure.push(
                params.0[0].to_string(),
                y.state.second_iteration,
            );

            let r = up_macro(|p, args, y, _trace| {
                let url = y.state.register_id(&p.0[0].to_string(), CrefKind::HSection, id_trace.clone())?;
                y.state.sticky_state.hsections.insert(p.0[0].to_string(), crate::HSectionInfo {
                    name: y.state.hsection_name[level].clone(),
                    title: args[0].to_string(),
                    numbering: numbering.clone(),
                });

                return Ok(format!(r###"<section>
    <h{} id="{}"><a href="{}">{}{}{}{}</a></h{}>
    {}
</section>"###,
                    level + 1,
                    p.0[0].to_string(),
                    url,
                    if y.state.hsection_render_number[level] && !no_numbering { &y.state.hsection_pre_number[level] } else { "" },
                    if y.state.hsection_render_number[level] && !no_numbering { &numbering } else { "" },
                    if y.state.hsection_render_number[level] && !no_numbering { &y.state.hsection_post_number[level] } else { "" },
                    args[0],
                    level + 1,
                    args[1]).into(),
                );
            }, &params, args, trace, y);

            for i in 0..6 {
                if i > level {
                    y.state.hsection_current_count[i] = 0;
                }
            }
            y.state.hsection_level -= 1;
            y.state.sticky_state.hsections_structure.pop();

            return r;
        }

        OutInternal::Toggled(trace, params, args, invisible, visible) => {
            arguments_exact(1, &args, &trace)?;
            return down_macro(|p, _n, _y, _trace| {
                return Ok(Out::Many(vec![
                        Out::Text(format!(r###"<button class="btn_toggle no" id="btn_toggle_{}">{}</button><div class="toggled" style="display: none">"###, p.0[0], invisible).into()),
                        Out::Argument(0),
                        Out::Text(format!(r###"</div>
    <script>
        (()=>{{
            let shown = false;
            const btn = document.querySelector("#btn_toggle_{}");
            const sol = btn.nextSibling;
            btn.addEventListener("click", e => {{
                sol.style.display = shown ? "none" : "block";
                btn.textContent = shown ? "{}" : "{}";
                btn.classList.toggle("yes", !shown);
                btn.classList.toggle("no", shown);
                shown = !shown;
            }});
        }})()
    </script>"###, p.0[0], invisible, visible).into()),
                    ]))
            }, &params, args, trace, y);
        }

        OutInternal::ChapterNav(trace, params, args) => {
            arguments_exact(0, &args, &trace)?;
            return down_macro(|_p, _n, y, trace| {
                if y.state.second_iteration {
                    let (previous, next) = y.state.sticky_state.hsections_structure.previous_and_next_ids();

                    let prev_link = if let Some(id) = previous {
                        let info = y.state.sticky_state.hsections.get(id).unwrap();
                        format!(r###"<a href="{}">{}</a>"###, y.state.resolve_id_to_url(id, trace.clone())?, info.title)
                    } else {
                        "".to_string()
                    };
                    let next_link = if let Some(id) = next {
                        let info = y.state.sticky_state.hsections.get(id).unwrap();
                        format!(r###"<a href="{}">{}</a>"###, y.state.resolve_id_to_url(id, trace.clone())?, info.title)
                    } else {
                        "".to_string()
                    };

                    let nav = format!(
                        r###"<nav class="chapter_navigation slightlywide">
    <div class="previous_chapter">
        {}
    </div>
    <div class="navigation_to_toc"><a href="{}">Table of Contents</a></div>
    <div class="next_chapter">
        {}
    </div>
</nav>"###,
                        prev_link,
                        y.state.resolve_id_to_url("toc", trace)?,
                        next_link,
                    );

                    return Ok(Out::Text(nav.into()));
                } else {
                    return Ok(Out::Many(vec![]))
                }
            }, &params, args, trace, y);
        }

        OutInternal::Box(trace, params, args, kind, name) => {
            arguments_gte(1, &args, &trace)?;
            arguments_lt(3, &args, &trace)?;

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
            let r = up_macro(|p, args, y, _trace| {
                let url = y.state.register_id(&id.clone(), CrefKind::Box, id_trace.clone())?;
                let classes = p.0.get(1).map(|s| s.to_string()).unwrap_or(String::new());
                y.state.sticky_state.boxes.insert(id.to_string(), crate::BoxInfo {
                    name: name.clone(),
                    numbering: numbering.clone(),
                    kind: kind.clone(),
                    classes: classes.clone(),
                });

                let box_html = format!(r###"<article class="{} {}" id="{}">
    <h6><a href="{}">{} {}{}{}</a></h6>
    {}
</article>"###,
                    kind.class(),
                    classes,
                    id,
                    url,
                    name,
                    numbering,
                    if args.len() == 2 { ": " } else { "" },
                    if args.len() == 2 { args[0].clone() } else { Rope::new() },
                    args[args.len() - 1],
                );

                y.state.create_preview(&id, &box_html)?;
                y.state.create_box_previews(&box_html)?;
                return Ok(box_html.into());
            }, &params, args, trace, y);

            y.state.box_current = None;

            return r;
        }

        OutInternal::Fact(trace, params, args, name, no_numbering) => {
            arguments_gte(1, &args, &trace)?;
            arguments_lt(3, &args, &trace)?;

            if !no_numbering {
                y.state.box_other_current_count += 1;
            }
            let hsection_level = y.state.box_other_level;
            let number = y.state.box_other_current_count;

            let mut numbering = " ".to_string();
            for i in 0..=hsection_level {
                if y.state.hsection_current_count[i] != 0 {
                    numbering.push_str(&format!("{}", y.state.hsection_current_count[i]));
                    numbering.push('.');
                }
            }
            numbering.push_str(&format!("{}", number));
            if no_numbering {
                numbering = "".to_string();
            }

            let id = params.0[0].clone();
            y.state.box_current = Some(id.to_string());

            let id_trace = Trace(None);
            let r = up_macro(|_p, args, y, _trace| {
                let url = y.state.register_id(&id.clone(), CrefKind::Box, id_trace.clone())?;
                y.state.sticky_state.boxes.insert(id.to_string(), crate::BoxInfo {
                    name: name.clone(),
                    numbering: numbering.clone(),
                    kind: BoxKind::fact(),
                    classes: "".to_string(),
                });

                let box_html = format!(r###"<article class="{}" id="{}">
    <h6><a href="{}">{}{}{}{}</a></h6>
    {}
</article>"###,
                    BoxKind::fact().class(),
                    id,
                    url,
                    name,
                    numbering,
                    if args.len() == 2 { ": " } else { "" },
                    if args.len() == 2 { args[0].clone() } else { Rope::new() },
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
            arguments_gte(1, &args, &trace)?;
            arguments_lt(2, &args, &trace)?;

            let kind = BoxKind::proof();
            let name = "Proof";

            let id = if params.0.len() == 1 {format!("proof_{}", params.0[0])} else {params.0[2].to_string()};
            y.state.box_current = Some(id.to_string());

            let id_trace = Trace(None);
            let r = up_macro(|p, args, y, _trace| {
                let url = y.state.register_id(&id.clone(), CrefKind::Box, id_trace.clone())?;
                y.state.sticky_state.boxes.insert(id.to_string(), crate::BoxInfo {
                    name: name.to_string(),
                    numbering: "".to_string(),
                    kind: kind.clone(),
                    classes: "".to_string(),
                });
                let claim_name = y.state.claim_name(&params.0[0], id_trace.clone())?;

                let box_html = format!(r###"<article class="{}" id="{}">
    <h6><a href="{}">{}Proof of {}</a></h6>
    {}
</article>"###,
                    kind.class(),
                    id,
                    url,
                    if p.0.len() > 1 {&p.0[1]} else {""},
                    claim_name,
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

                                CrefKind::Case => {
                                    let numbering = y.state.sticky_state.cases.get(&id.to_string()).unwrap();
                                    let label = if args.len() == 2 {
                                        args[1].to_string()
                                    } else {
                                        format!("Case&nbsp;{}", numbering)
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
                                            if y.state.mathmode {
                                                let label = if args.len() == 2 {
                                                    args[1].to_string()
                                                } else {
                                                    format!("{}~{}", box_info.name, box_info.numbering)
                                                };

                                                let tex = format!(
                                                    r###"\href{{{}}}{{\htmlClass{{ref {}}}{{\htmlData{{preview={}, width={}}}{{{}}}}}}}"###,
                                                    url,
                                                    box_info.kind.class(),
                                                    y.state.id_to_preview_url(id),
                                                    box_info.classes,
                                                    label,
                                                );
                                                return Ok(tex.into());
                                            } else {
                                                let label = if args.len() == 2 {
                                                    args[1].to_string()
                                                } else {
                                                    format!("{}&nbsp;{}", box_info.name, box_info.numbering)
                                                };

                                                let tag = format!(
                                                    r###"<a class="ref {}" href="{}" data-preview="{}" data-width="{}">{}</a>"###,
                                                    box_info.kind.class(),
                                                    url,
                                                    y.state.id_to_preview_url(id),
                                                    box_info.classes,
                                                    label,
                                                );
                                                return Ok(tag.into());
                                            }
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

            return up_macro(|p, args, y, trace| {
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

                let defined = if p.0.len() == 2 {
                    p.0[1].to_string()
                } else {
                    args[0].to_string()
                };
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

                y.state.register_define(defined, href.clone(), preview_url.clone(), singular, plural, trace)?;

                return Ok(format!(r###"<dfn{} data-preview="{}"><a href="{}">{}</a></dfn>"###,
                    if boxless { format!(r#" id="{}""#, target_id) } else { "".to_string() },
                    preview_url,
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
                                if y.state.mathmode {
                                    return Ok(format!(
                                        r###"\href{{{}}}{{\htmlClass{{ref definition}}{{\htmlData{{preview={}}}{{{}}}}}}}"###,
                                        info.href,
                                        y.state.resolve_defined_to_preview_url(id, id_trace.clone())?,
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

        OutInternal::Enclose2(trace, params, args, pre, mid, post) => {
            arguments_exact(2, &args, &trace)?;

            return down_macro(|_p, _n, _y, _trace| {
                return Ok(Out::Many(vec![
                        Out::Text(pre.into()),
                        Out::Argument(0),
                        Out::Text(mid.into()),
                        Out::Argument(1),
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

        OutInternal::MathMacro(trace, params, args, math_id, tex) => {
            arguments_exact(0, &args, &trace)?;

            if y.state.second_iteration {
                match y.state.sticky_state.math_definitions.get(&math_id) {
                    None => return Err(ExpansionError::UnknownMathId(trace, math_id.to_string())),
                    Some(id) => {
                        let url = y.state.resolve_id_to_url(id, trace)?;
                        let preview_url = y.state.id_to_preview_url(id);

                        if params.0[0] {
                            return Ok(format!(r###"\htmlData{{preview={}}}{{{}}}"###, preview_url, tex).into());
                        } else {
                            return Ok(format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{{}}}}}"###, preview_url, url, tex).into());
                        }
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

        OutInternal::EncloseFunctionApplication(trace, params, args, math_id, name) => {
            let len = args.len();
            return down_macro(|p, _n, y, trace| {
                if y.state.second_iteration {
                    match y.state.sticky_state.math_definitions.get(&math_id) {
                        None => return Err(ExpansionError::UnknownMathId(trace, math_id.to_string())),
                        Some(id) => {
                            let url = y.state.resolve_id_to_url(id, trace)?;
                            let preview_url = y.state.id_to_preview_url(id);

                            let name_tex = format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{{}}}}}"###, preview_url, url, name);

                            let outs = (0..len).map(|i| Out::Argument(i)).collect();
                            return Ok(Out::Many(vec![
                                    Out::Text(name_tex.into()),
                                    Out::MathFunctionParens(p.clone(), outs)
                                ]));
                        }
                    }
                } else {
                    return Ok(Out::Argument(0));
                }
            }, &params, args, trace, y);
        }

        OutInternal::MathSet(trace, params, args) => {
            return down_macro(|p, n, y, trace| {
                if y.state.second_iteration {
                    match y.state.sticky_state.math_definitions.get("set") {
                        None => return Err(ExpansionError::UnknownMathId(trace, "set".to_string())),
                        Some(id) => {
                            let url = y.state.resolve_id_to_url(id, trace)?;
                            let preview_url = y.state.id_to_preview_url(id);
                            if n == 0 {
                                return Ok(Out::Text(format!(r###"\htmlData{{preview={}}}{{\href{{{}}}{{\emptyset}}}}"###, preview_url, url).into()));
                            } else {
                                let (sizing_left, sizing_right) = sizing_level(p.0[0]);
                                let left_delimiter = format!(r###" {}\lbrace "###, sizing_left);
                                let right_delimiter = format!(r###" {}\rbrace "###, sizing_right);

                                let mut outs = vec![Out::Text(left_delimiter.into())];
                                for i in 0..n {
                                    if i != 0 {
                                        outs.push(Out::Text(r###", \allowbreak"###.into()));
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

        OutInternal::MathSetBuilder(trace, params, args) => {
            arguments_exact(2, &args, &trace)?;

            let (sizing_left, sizing_right) = sizing_level(params.0[0]);
            return down_macro(|_p, _n, _y, _trace| Ok(Out::Many(vec![
                    Out::Text(format!(r###"{}\lbrace "###, sizing_left).into()),
                    Out::Argument(0),
                    Out::Text(r###"\mid"###.into()),
                    Out::Argument(1),
                    Out::Text(format!(r###" {}\rbrace "###, sizing_right).into()),
                ])), &params, args, trace, y);
        }

        OutInternal::MathGroupingParens(trace, params, args)
        | OutInternal::MathFunctionParens(trace, params, args) => {
            arguments_exact(1, &args, &trace)?;
            return down_macro(|p, n, y, trace| {
                let (sizing_left, sizing_right) = sizing_level(p.0[0]);
                let left_delimiter = format!(r###" {}( "###, sizing_left);
                let right_delimiter = format!(r###" {}) "###, sizing_right);

                return Ok(Out::Many(vec![
                        Out::Text(left_delimiter.into()),
                        Out::Argument(0),
                        Out::Text(right_delimiter.into()),
                    ]));
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

        OutInternal::Cases(trace, params, args) => {
            arguments_exact(1, &args, &trace)?;
            y.state.cases.push(0);
            let r = down_macro(|_p, _n, _y, _trace| Ok(Out::Many(vec![
                    Out::Text(r###"<div class="cases">"###.into()),
                    Out::Argument(0),
                    Out::Text("</div>".into()),
                ])), &params, args, trace, y)?;
            let _ = y.state.cases.pop();
            return Ok(r);
        }

        OutInternal::Case(trace, params, args) => {
            arguments_gte(1, &args, &trace)?;
            arguments_lt(3, &args, &trace)?;

            return down_macro(|p, n, y, trace| {
                match y.state.cases.last_mut() {
                    Some(last) => *last += 1,
                    None => return Err(ExpansionError::CaseNotInCases(trace.clone())),
                }

                let mut numbering = "".to_string();
                for i in 0..y.state.cases.len() {
                    if i != 0 {
                        numbering.push('.');
                    }
                    numbering.push_str(&format!("{}", y.state.cases[i]))
                }

                let id = &p.0[0];
                let linkable = id != "";
                let href = if linkable {
                    y.state.sticky_state.cases.insert(id.to_string(), numbering.to_string());
                    y.state.register_id(id, CrefKind::Case, trace.clone())?
                } else {
                    "".to_string()
                };

                return Ok(Out::Many(vec![
                    Out::Text(r###"<div class="proof_part_title"><span class="case_name">"###.into()),
                    Out::Text(
                        if linkable {
                            format!(r###"<a href="{}" id="{}">"###, href, id).into()
                        } else {
                            "".into()
                        }
                    ),
                    Out::Text(format!("Case {}:</span> ", numbering).into()),
                    if n >= 2 {Out::Argument(0)} else {Out::Text("".into())},
                    Out::Text(if linkable {"</a>".into()} else {"".into()}),
                    Out::Text("</div>".into()),
                    Out::Text(r###"<div class="proof_part_body">"###.into()),
                    Out::Argument(n - 1),
                    Out::Text("</div>".into()),
                ]));
            }, &params, args, trace, y);
        }

        OutInternal::Drop(trace, params, args) => {
            return up_macro(|_p, _args, _y, _trace| Ok("".into()), &params, args, trace, y);
        }

        OutInternal::ProofPart(trace, params, args) => {
            arguments_exact(2, &args, &trace)?;
            return down_macro(|_p, _n, _y, _trace| Ok(Out::Many(vec![
                    Out::Text(r###"<div class="proof_part"><div class="proof_part_title">"###.into()),
                    Out::Argument(0),
                    Out::Text(r###"</div><div class="proof_part_body">"###.into()),
                    Out::Argument(1),
                    Out::Text("</div></div>".into()),
                ])), &params, args, trace, y);
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

        Out::MathFunctionParens(params, a) => return Ok(OutInternal::MathFunctionParens(Trace(None), params, outs_to_internals(a, args)?)),
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

fn sizing_level(level: u8) -> (String, String) {
    match level {
        99 => (r###"\left"###.to_string(), r###"\right"###.to_string()),
        0 => ("".to_string(), "".to_string()),
        1 => (r###"\big"###.to_string(), r###"\big"###.to_string()),
        2 => (r###"\Big"###.to_string(), r###"\Big"###.to_string()),
        3 => (r###"\bigg"###.to_string(), r###"\bigg"###.to_string()),
        4 => (r###"\Bigg"###.to_string(), r###"\Bigg"###.to_string()),
        _ => panic!("invalid delimiter sizing"),
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
pub struct HSection([String; 1]);

impl Default for HSection {
    fn default() -> Self {
        HSection(["".to_string()])
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
pub struct BoxParams(Vec<String>);

impl Default for BoxParams {
    fn default() -> Self {
        BoxParams(vec!["".to_string()])
    }
}

#[derive(Deserialize, Clone)]
pub struct Proof(Vec<String>);

impl Default for Proof {
    fn default() -> Self {
        Proof(Vec::new())
    }
}

#[derive(Deserialize, Clone)]
pub struct Define(Vec<String>);

impl Default for Define {
    fn default() -> Self {
        Define(vec!["".to_string()])
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

#[derive(Deserialize, Clone)]
pub struct Case([String; 1]);

impl Default for Case {
    fn default() -> Self {
        Case(["".to_string()])
    }
}

#[derive(Deserialize, Clone)]
pub struct MathMacro([bool; 1]);

impl Default for MathMacro {
    fn default() -> Self {
        MathMacro([false])
    }
}

#[derive(Deserialize, Clone)]
pub struct Toggled([String; 1]);

impl Default for Toggled {
    fn default() -> Self {
        Toggled(["".to_string()])
    }
}

#[derive(Deserialize, Clone)]
pub struct MathSet([u8; 1]);

impl Default for MathSet {
    fn default() -> Self {
        MathSet([99])
    }
}

#[derive(Deserialize, Clone)]
pub struct MathSetBuilder([u8; 1]);

impl Default for MathSetBuilder {
    fn default() -> Self {
        MathSetBuilder([99])
    }
}
