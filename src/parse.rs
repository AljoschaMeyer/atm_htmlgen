use sourcefile::SourceFile;
use thiserror::Error;
use ropey::Rope;
use serde::Deserialize;

use atm_parser_helper::{Eoi, ParserHelper};
use valuable_value::human::{VVDeserializer, Error as VVError};

use crate::{Yatt, print_trace, BoxKind};
use crate::macros::{OutInternal, Trace};

// An offset into the source map.
pub(crate) type Offset = usize;

pub(crate) type OffsetSpan = (Offset /* start, inclusive */, Offset /* end, exclusive */);

#[derive(Error, Debug)]
pub(crate) enum ParseError {
    #[error("never printed")]
    Eoi,
    #[error("never printed")]
    Parameters(VVError, Trace),
    #[error("never printed")]
    UnknownMacroName(Trace),
}

impl ParseError {
    pub fn print_parse_error(&self, source: &SourceFile) {
        match self {
            ParseError::Eoi => {
                println!("Unexpected end of input.");
            }
            ParseError::UnknownMacroName(t) => {
                print!("Unknown macro name.");
                print_trace(t.clone(), source, false);
            }
            ParseError::Parameters(e, t) => {
                print!("Could not parse macro parameters.");
                print_trace(t.clone(), source, true);
                println!("\n{}", e);
            }
        }
    }
}

impl Eoi for ParseError {
    fn eoi() -> Self {
        ParseError::Eoi
    }
}

pub(crate) fn parse(s: &str, y: &mut Yatt, source_offset: Offset) -> Result<OutInternal, ParseError> {
    let mut p = Parser {
        p: ParserHelper::new(s.as_bytes()),
    };
    return p.parse(y, source_offset);
}

struct Parser<'a> {
    p: ParserHelper<'a>,
}

impl<'a> Parser<'a> {
    fn parse(&mut self, y: &mut Yatt, source_offset: Offset) -> Result<OutInternal, ParseError> {
        let initial_position = self.p.position();

        // Skip over first whitespace without adding it to the output rope.
        loop {
            if self.p.advance_over("§#".as_bytes()) { // `§#` initiates a line comment
                loop {
                    match self.p.next_or_end() {
                        Some(0x0a) | None => break,
                        Some(_) => {}
                    }
                }
            } else {
                match self.p.peek_or_end() {
                    Some(0x09) | Some(0x0a) | Some(0x0d) | Some(0x20) => self.p.advance(1),
                    Some(_) | None => break,
                }
            }
        }

        let mut outs = Vec::new();
        let mut rope = Rope::new();
        let mut start = self.p.position();
        let mut last_non_ws = self.p.position();
        let mut pending_parens = 1;

        loop {
            if self.p.rest().len() == 0 {
                let text = unsafe { std::str::from_utf8_unchecked(self.p.slice(start..last_non_ws)) };
                rope.insert(rope.len_chars(), text);
                let trace = Trace(Some((source_offset, source_offset + self.p.position() - initial_position)));
                outs.push(OutInternal::Text(rope.clone(), trace));
                break;
            } else if self.p.advance_over("§".as_bytes()) {
                let text = unsafe { std::str::from_utf8_unchecked(self.p.slice(start..self.p.position() - 2)) };
                rope.insert(rope.len_chars(), text);
                let trace = Trace(Some((source_offset, source_offset + self.p.position() - initial_position)));
                outs.push(OutInternal::Text(rope.clone(), trace));
                rope = Rope::new();

                if self.p.advance_over("§".as_bytes()) {
                    rope.insert(rope.len_chars(), "§");
                    start = self.p.position();
                    last_non_ws = self.p.position();
                } else if self.p.advance_over("(".as_bytes()) {
                    rope.insert(rope.len_chars(), "(");
                    start = self.p.position();
                    last_non_ws = self.p.position();
                } else if self.p.advance_over(")".as_bytes()) {
                    rope.insert(rope.len_chars(), ")");
                    start = self.p.position();
                    last_non_ws = self.p.position();
                } else if self.p.advance_over("#".as_bytes()) {
                    loop {
                        match self.p.next_or_end() {
                            Some(0x0a) | None => {
                                start = self.p.position();
                                break;
                            }
                            Some(_) => {}
                        }
                    }
                } else {
                    let trace_start = source_offset + self.p.position() - initial_position;
                    let start_macro_name = self.p.position();

                    loop {
                        if
                            self.p.rest().starts_with("§".as_bytes()) ||
                            self.p.rest().starts_with(" ".as_bytes()) ||
                            self.p.rest().starts_with("\n".as_bytes()) ||
                            self.p.rest().starts_with("\t".as_bytes()) ||
                            self.p.rest().starts_with("\r".as_bytes()) ||
                            self.p.rest().starts_with("[".as_bytes()) ||
                            self.p.rest().starts_with("{".as_bytes()) ||
                            self.p.rest().starts_with("(".as_bytes()) ||
                            self.p.rest().starts_with(")".as_bytes()) ||
                            self.p.rest().len() == 0
                        {
                            break;
                        } else {
                            self.p.advance(1);
                        }
                    }

                    let parse_parameters = self.p.rest().starts_with("[".as_bytes()) || self.p.rest().starts_with("{".as_bytes());
                    let macro_name = self.p.slice(start_macro_name..self.p.position());

                    if macro_name == b"" {
                        self.pm(OutInternal::EmptyMacro, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;

                        self.p.advance(1);
                        start = self.p.position();
                    } else if macro_name == b"html" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "html".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"style" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "style".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"title" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "title".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"body" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "body".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"head" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "head".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"aside" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "aside".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"footer" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "footer".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"header" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "header".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"h1" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "h1".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"h2" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "h2".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"h3" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "h3".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"h4" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "h4".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"h5" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "h5".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"h6" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "h6".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"main" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "main".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"nav" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "nav".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"section" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "section".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"blockquote" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "blockquote".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"figcaption" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "figcaption".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"figure" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "figure".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"pre" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "pre".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"div" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "div".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"p" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "p".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"li" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "li".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"ul" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "ul".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"ol" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "ol".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"a" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "a".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"abbr" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "abbr".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"code" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "code".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"dfn" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "dfn".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"em" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "em".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"q" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "q".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"s" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "s".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"span" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "span".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"strong" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "strong".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"script" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "script".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"input" {
                        self.pm(OutInternal::Input, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"output" {
                        self.pm(|t, p, a| OutInternal::Output(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"output_tee" {
                        self.pm(|t, p, a| OutInternal::Output(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"copy" {
                        self.pm(OutInternal::CopyAll, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"template" {
                        self.pm(OutInternal::Template, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"hsection" {
                        self.pm(OutInternal::HSection, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lorem" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, LOREM), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"cref" {
                        self.pm(OutInternal::Cref, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$" {
                        self.pm(|t, p, a| OutInternal::TeX(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$$" {
                        self.pm(|t, p, a| OutInternal::TeX(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"cwd" {
                        self.pm(OutInternal::Cwd, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"set_domain" {
                        self.pm(OutInternal::SetDomain, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"definition" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::definition(), "Definition".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"example" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::example(), "Example".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"exercise" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::exercise(), "Exercise".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"statement" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::fact(), "Statement".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"observation" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::fact(), "Observation".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"theorem" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::fact(), "Theorem".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lemma" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::fact(), "Lemma".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"corollary" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::fact(), "Corollary".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"conjecture" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::fact(), "Conjecture".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"colorExercise" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, COLOR_EXERCISE), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"colorFact" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, COLOR_FACT), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"colorExample" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, COLOR_EXAMPLE), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"colorDefinition" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, COLOR_DEFINITION), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"define" {
                        self.pm(OutInternal::Define, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"r" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, false, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"R" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, true, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rs" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, false, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"Rs" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, true, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else {
                        let trace_end = source_offset + self.p.position() - initial_position;
                        let trace = Trace(Some((trace_start, trace_end)));
                        return Err(ParseError::UnknownMacroName(trace));
                    }

                    last_non_ws = self.p.position();
                }
            } else {
                let c = self.p.next::<ParseError>().unwrap();

                if c == ('(' as u8) {
                    pending_parens += 1;
                    last_non_ws = self.p.position();
                } else if c == (')' as u8) {
                    pending_parens -= 1;

                    if pending_parens == 0 {
                        let text = unsafe { std::str::from_utf8_unchecked(self.p.slice(start..last_non_ws)) };
                        rope.insert(rope.len_chars(), text);
                        let trace = Trace(Some((source_offset, source_offset + self.p.position() - initial_position)));
                        outs.push(OutInternal::Text(rope.clone(), trace));
                        break;
                    } else {
                        last_non_ws = self.p.position();
                    }
                } else if c == (' ' as u8) || c == ('\n' as u8) || c == ('\t' as u8) || c == ('\r' as u8) {
                    // noop
                } else {
                    last_non_ws = self.p.position();
                }
            }
        }

        if outs.len() == 1 {
            return Ok(outs.pop().unwrap());
        } else {
            return Ok(OutInternal::Many(outs.into()));
        }
    }

    fn parse_args(&mut self, y: &mut Yatt, source_offset: Offset) -> Result<Vec<OutInternal>, ParseError> {
        let initial_position = self.p.position();
        let mut outs = Vec::new();

        loop {
            if self.p.advance_over(b"(") {
                outs.push(self.parse(y, source_offset + self.p.position() - initial_position)?);
            } else {
                return Ok(outs.into());
            }
        }
    }

    // Parse macro.
    fn pm<'s, P, M>(&'s mut self, m: M, y: &mut Yatt, source_offset: Offset, parse_parameters: bool, initial_position: usize, trace_start: usize, outs: &mut Vec<OutInternal>, start: &mut usize, last_non_ws: &mut usize) -> Result<(), ParseError>
    where
        P: Deserialize<'s> + Default,
        M: Fn(Trace, P, Vec<OutInternal>) -> OutInternal,
    {
        let params = if parse_parameters {
            let mut d = VVDeserializer::new(self.p.rest());
            match <P>::deserialize(&mut d) {
                Ok(v) => {
                    self.p.advance(d.position());
                    v
                }
                Err(e) => {
                    self.p.advance(d.position());
                    let trace_end = source_offset + self.p.position() - initial_position;
                    let trace = Trace(Some((trace_start, trace_end)));
                    return Err(ParseError::Parameters(e, trace));
                }
            }
        } else {
            <P>::default()
        };

        let args = self.parse_args(y, source_offset + self.p.position() - initial_position)?;

        let trace_end = source_offset + self.p.position() - initial_position;
        let trace = Trace(Some((trace_start, trace_end)));

        outs.push(m(trace, params, args));
        *start = self.p.position();
        *last_non_ws = self.p.position();

        return Ok(());
    }
}

static LOREM: &str = "Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla consequat massa quis enim.";

static COLOR_FACT: &str = "rgb(204, 115, 0)";
static COLOR_DEFINITION: &str = "rgb(124, 0, 132)";
static COLOR_EXAMPLE: &str = "rgb(0, 133, 18)";
static COLOR_EXERCISE: &str = "rgb(0, 95, 133)";