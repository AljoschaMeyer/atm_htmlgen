use sourcefile::SourceFile;
use thiserror::Error;
use ropey::Rope;
use serde::Deserialize;

use atm_parser_helper::{Eoi, ParserHelper};
use valuable_value::human::{VVDeserializer, Error as VVError};

use crate::{Yatt, print_trace, BoxKind};
use crate::macros::{OutInternal, Trace};
use crate::set_examples::{S1, S2, S3, Operator, Term};
use Operator::*;

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
                    // } else if macro_name == b"figure" {
                    //     self.pm(|t, p, a| OutInternal::HtmlTag(t, "figure".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"pre" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "pre".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"div" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "div".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"p" {
                        self.pm(|t, p, a| OutInternal::P(t, p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"li" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "li".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"ul" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "ul".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"ol" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "ol".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"button" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "button".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"a" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "a".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"abbr" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "abbr".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"code" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "code".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"dfn" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "dfn".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"html_cite" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "cite".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"em" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "em".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"i" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "i".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"b" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "b".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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
                    } else if macro_name == b"table" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "table".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"thead" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "thead".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"tbody" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "tbody".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"tr" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "tr".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"th" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "th".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"td" {
                        self.pm(|t, p, a| OutInternal::HtmlTag(t, "td".to_string(), p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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
                        self.pm(|t, p, a| OutInternal::HSection(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"hsection*" {
                        self.pm(|t, p, a| OutInternal::HSection(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"drop" {
                        self.pm(|t, p, a| OutInternal::Drop(t, p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"side" {
                        self.pm(|t, p, a| OutInternal::Aside(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"side*" {
                        self.pm(|t, p, a| OutInternal::Aside(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"pretty_venn_duration" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"60s"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lorem" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, LOREM), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"meta1" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<i class="bgclll1 highlight low">bla</i>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"meta2" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<i class="bgclll3 highlight low">blubb</i>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"meta3" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<i class="bgclll5 highlight low">blip</i>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$mid" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\mid"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol0" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container">/span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol0" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\char"e904}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol1" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol1" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\char"e903}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol2" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol2" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\char"e902}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol3" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol3" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\char"e901}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol4" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol4" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\char"e900}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"hr" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, "<hr>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"br" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, "<br>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$ldots" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, "\\ldots"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_svg" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, EULER_SVG), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_svg_union" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, EULER_SVG_UNION), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_svg_setminus" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, EULER_SVG_SETMINUS), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_svg_equality" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, EULER_SVG_EQUALITY), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_svg_intersection" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, EULER_SVG_INTERSECTION), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_toggles" {
                        self.pm(OutInternal::EulerToggles, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"euler_toggles_power" {
                        self.pm(OutInternal::EulerTogglesPower, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"cref" {
                        self.pm(OutInternal::Cref, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"cases" {
                        self.pm(OutInternal::Cases, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"case" {
                        self.pm(OutInternal::Case, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"chapternav" {
                        self.pm(OutInternal::ChapterNav, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$" {
                        self.pm(|t, p, a| OutInternal::TeX(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$$" {
                        self.pm(|t, p, a| OutInternal::TeX(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"fleqn" {
                        self.pm(|t, p, a| OutInternal::Fleqn(t, p, a), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"cwd" {
                        self.pm(OutInternal::Cwd, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"set_domain" {
                        self.pm(OutInternal::SetDomain, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"definition" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::definition(), "Definition".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"example" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::example(), "Example".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"figure" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::example(), "Figure".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"exercise" {
                        self.pm(|t, p, a| OutInternal::Box(t, p, a, BoxKind::exercise(), "Exercise".to_string()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"statement" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Statement".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"observation" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Observation".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"theorem" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Theorem".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lemma" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Lemma".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"corollary" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Corollary".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"conjecture" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Conjecture".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"falsehood" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Falsehood".to_string(), false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"statement*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Statement".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"observation*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Observation".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"theorem*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Theorem".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lemma*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Lemma".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"corollary*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Corollary".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"conjecture*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Conjecture".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"falsehood*" {
                        self.pm(|t, p, a| OutInternal::Fact(t, p, a, "Falsehood".to_string(), true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"proof" {
                        self.pm(OutInternal::Proof, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"define" {
                        self.pm(|t, p, a| OutInternal::Define(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"definex" {
                        self.pm(|t, p, a| OutInternal::Define(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"proof_part" {
                        self.pm(OutInternal::ProofPart, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"css_colors" {
                        self.pm(OutInternal::CssColors, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"powerset_colors" {
                        self.pm(OutInternal::PowersetColors, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"r" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, false, false, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"R" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, true, false, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rs" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, false, true, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"Rs" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, true, true, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rdef" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, false, false, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"Rdef" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, true, false, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rsdef" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, false, true, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"Rsdef" {
                        self.pm(|t, p, a| OutInternal::ReferenceDefined(t, p, a, true, true, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"proven_fact" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<div class="proven_fact">"###, "</div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"clfx" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<div class="clfx">"###, "</div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lparen" {
                        self.pm(|t, p, a| OutInternal::LeftDelimiter(t, p, a, "("), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rparen" {
                        self.pm(|t, p, a| OutInternal::RightDelimiter(t, p, a, ")"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lquote" {
                        self.pm(|t, p, a| OutInternal::LeftDelimiter(t, p, a, "“"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rquote" {
                        self.pm(|t, p, a| OutInternal::RightDelimiter(t, p, a, "”"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"parens" {
                        self.pm(|t, p, a| OutInternal::TextDelimiters(t, p, a, "(", ")"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"verbatim" {
                        self.pm(|t, p, a| OutInternal::TextDelimiters(t, p, a, "“", "”"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$value" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\mathrm{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$cancel" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\cancel{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$bcancel" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\bcancel{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$xcancel" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\xcancel{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$sout" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\sout{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$cancel_symbol" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"\htmlClass{cancel_symbol}{"###, "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$define_notation" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"\htmlClass{define_notation}{"###, "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$text_color" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"\htmlClass{text_color}{"###, "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"nowrap" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="nowrap">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"video_container" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<div class="video_container">"###, "</div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"solution" {
                        self.pm(|t, p, a| OutInternal::Toggled(t, p, a, r###"Show Solution"###, "Hide Solution"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"proof_as_exercise" {
                        self.pm(|t, p, a| OutInternal::Toggled(t, p, a, r###"Show Proof"###, "Hide Proof"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"nobr" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="nobr">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$tag" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\tag{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$p" {
                        self.pm(OutInternal::MathGroupingParens, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$text" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\text{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"highlightlow1" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="bgclll1 highlight low">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlow1" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll1 highlight low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttop1" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll1 highlight top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"highlightlow2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="bgclll2 highlight low">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlow2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll2 highlight low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttop2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll2 highlight top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"highlightlow3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="bgclll3 highlight low">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlow3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll3 highlight low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttop3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll3 highlight top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"highlightlow4" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="bgclll4 highlight low">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlow4" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll4 highlight low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttop4" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll4 highlight top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"highlightlow5" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="bgclll5 highlight low">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlow5" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll5 highlight low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttop5" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll5 highlight top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"highlightlow6" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="bgclll6 highlight low">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlow6" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll6 highlight low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttop6" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll6 highlight top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlowr1" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll1 highlight rightspace low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttopr1" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll1 highlight rightspace top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlowr2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll2 highlight rightspace low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttopr2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll2 highlight rightspace top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlowr3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll3 highlight rightspace low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttopr3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll3 highlight rightspace top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlowr4" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll4 highlight rightspace low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttopr4" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll4 highlight rightspace top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlowr5" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll5 highlight rightspace low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttopr5" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll5 highlight rightspace top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlightlowr6" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll6 highlight rightspace low}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlighttopr6" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgclll6 highlight rightspace top}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight1" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmclll1}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmclll2}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmclll3}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight4" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmclll4}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight5" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmclll5}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight6" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmclll6}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight1_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmcllldirect1}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight2_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmcllldirect2}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight3_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmcllldirect3}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight4_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmcllldirect4}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight5_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmcllldirect5}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight6_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{bgmcllldirect6}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$class" {
                        self.pm(|t, p, a| OutInternal::Enclose2(t, p, a, "\\htmlClass{", "}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"fact_marginalia" {
                        self.pm(|t, p, a| OutInternal::Enclose2(t, p, a, r###"<div class="box_marginalia fact clfx slightlywide"><span class="aside">"###, "</span>", "</div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"exercise_marginalia" {
                        self.pm(|t, p, a| OutInternal::Enclose2(t, p, a, r###"<div class="box_marginalia exercise clfx slightlywide"><span class="aside">"###, "</span>", "</div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"definition_marginalia" {
                        self.pm(|t, p, a| OutInternal::Enclose2(t, p, a, r###"<div class="box_marginalia definition clfx slightlywide"><span class="aside">"###, "</span>", "</div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"link" {
                        self.pm(OutInternal::Link, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"captioned" {
                        self.pm(OutInternal::Captioned, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"set_math_id" {
                        self.pm(OutInternal::SetMathId, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"set_tag" {
                        self.pm(|t, p, a| OutInternal::SetTag(t, p, a, false), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"set_tagc" {
                        self.pm(|t, p, a| OutInternal::SetTag(t, p, a, true), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"rtag" {
                        self.pm(OutInternal::RTag, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$set" {
                        self.pm(OutInternal::MathSet, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$set_builder" {
                        self.pm(OutInternal::MathSetBuilder, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$$align*" {
                        self.pm(|t, p, a| OutInternal::MathEnv(t, p, a, "align*".into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$eq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "eq".into(), r###"="###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$neq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "neq".into(), r###"\neq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$leq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "leq".into(), r###"\leq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$implies" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "implies".into(), r###"\implies"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$impliedby" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "impliedby".into(), r###"\impliedby"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$iff" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "iff".into(), r###"\iff"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$in" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "in".into(), r###"\in"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$notin" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "notin".into(), r###"\notin"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$seq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "seq".into(), r###"="###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$sneq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "sneq".into(), r###"\neq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$defeq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "defeq".into(), r###"\coloneqq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$eqdef" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "eqdef".into(), r###"\eqqcolon"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$subseteq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "subseteq".into(), r###"\subseteq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$subset" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "subset".into(), r###"\subset"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$supseteq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "supseteq".into(), r###"\supseteq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$supset" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "supset".into(), r###"\supset"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$nsubseteq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "nsubseteq".into(), r###"\nsubseteq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$nsupseteq" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "nsupseteq".into(), r###"\nsupseteq"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$nsubset" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "nsubset".into(), r###"\not\subset"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$nsupset" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "nsupset".into(), r###"\not\supset"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$intersection" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "intersection".into(), r###"\cap"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$union" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "union".into(), r###"\cup"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$setminus" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "setminus".into(), r###"\setminus"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symdif" {
                        self.pm(|t, p, a| OutInternal::MathMacro(t, p, a, "symdif".into(), r###"\operatorname{\triangle}"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$twice" {
                        self.pm(|t, p, a| OutInternal::EncloseFunctionApplication(t, p, a, "twice".into(), r###"\operatorname{twice}"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$powerset" {
                        self.pm(|t, p, a| OutInternal::EncloseFunctionApplication(t, p, a, "powerset".into(), r###"\operatorname{\mathcal{P}}"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_associative_intersection" {
                        self.pm(|t, p, a| OutInternal::EquationVenn3(t, p, a,
                            Term::Binary(
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S1)),
                                    Intersection,
                                    Box::new(Term::Unary(S2)),
                                )),
                                Intersection,
                                Box::new(Term::Unary(S3)),
                            ),
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Intersection,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S2)),
                                    Intersection,
                                    Box::new(Term::Unary(S3)),
                                )),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_associative_union" {
                        self.pm(|t, p, a| OutInternal::EquationVenn3(t, p, a,
                            Term::Binary(
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S1)),
                                    Union,
                                    Box::new(Term::Unary(S2)),
                                )),
                                Union,
                                Box::new(Term::Unary(S3)),
                            ),
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Union,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S2)),
                                    Union,
                                    Box::new(Term::Unary(S3)),
                                )),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_absorption_intersection_union" {
                        self.pm(|t, p, a| OutInternal::EquationVenn2(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Intersection,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S1)),
                                    Union,
                                    Box::new(Term::Unary(S2)),
                                )),
                            ),
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Union,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S1)),
                                    Intersection,
                                    Box::new(Term::Unary(S2)),
                                )),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_intersection_via_set_difference" {
                        self.pm(|t, p, a| OutInternal::EquationVenn2(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Intersection,
                                Box::new(Term::Unary(S2)),
                            ),
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Difference,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S1)),
                                    Difference,
                                    Box::new(Term::Unary(S2)),
                                )),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_exercise_set_difference2" {
                        self.pm(|t, p, a| OutInternal::EquationVenn3(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S3)),
                                Difference,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S2)),
                                    Difference,
                                    Box::new(Term::Unary(S1)),
                                )),
                            ),
                            Term::Binary(
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S3)),
                                    Intersection,
                                    Box::new(Term::Unary(S1)),
                                )),
                                Union,
                                Box::new(Term::Binary(
                                    Box::new(Term::Unary(S3)),
                                    Difference,
                                    Box::new(Term::Unary(S2)),
                                )),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_intersection" {
                        self.pm(|t, p, a| OutInternal::Venn2(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Intersection,
                                Box::new(Term::Unary(S2)),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_union" {
                        self.pm(|t, p, a| OutInternal::Venn2(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Union,
                                Box::new(Term::Unary(S2)),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_setdifference" {
                        self.pm(|t, p, a| OutInternal::Venn2(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                Difference,
                                Box::new(Term::Unary(S2)),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"venn_symmetric_difference" {
                        self.pm(|t, p, a| OutInternal::Venn2(t, p, a,
                            Term::Binary(
                                Box::new(Term::Unary(S1)),
                                SymmetricDifference,
                                Box::new(Term::Unary(S2)),
                            ),
                        ), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"polar_x" {
                        self.pm(OutInternal::PolarX, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"polar_y" {
                        self.pm(OutInternal::PolarY, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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

static EULER_SVG: &str = r###"<svg version="1.1" viewBox="-100 -100 200 200" xmlns="http://www.w3.org/2000/svg" class="eulersvg">
<path class="cd1 bgclll1" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path class="cd3 bgclll3" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>

<text x="0" y="-70">&#xe904;</text>
<text x="66.57" y="-21.6311">&#xe903;</text>
<text x="41.1449" y="56.6311">&#xe902;</text>
<text x="-41.1449" y="56.6311">&#xe901;</text>
<text x="-66.57" y="-21.6311">&#xe900;</text>
</svg>"###;

static EULER_SVG_EQUALITY: &str = r###"<svg version="1.1" viewBox="-100 -100 200 200" xmlns="http://www.w3.org/2000/svg" class="eulersvg">
<path class="cd1 bgclll1" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path class="cd3 bgclll3" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>

<text x="0" y="-70">&#xe904;</text>
<text x="66.57" y="-21.6311">&#xe903;</text>
<text class="obstruction" x="41.1449" y="56.6311">&#xe902;</text>
<text class="obstruction" x="-41.1449" y="56.6311">&#xe901;</text>
<text class="obstruction" x="-66.57" y="-21.6311">&#xe900;</text>
</svg>"###;

static EULER_SVG_INTERSECTION: &str = r###"<svg version="1.1" viewBox="-100 -100 200 200" xmlns="http://www.w3.org/2000/svg" class="eulersvg">
<path class="cd1 bgclll1" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path class="cd3 bgclll3" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>

<clipPath id="intersection_clip1_euler">
<path id="intersection_clip1_euler_path" class="euler clip" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
</clipPath>
<clipPath id="intersection_clip2_euler" clip-path="url('#intersection_clip1_euler')">
<path id="intersection_clip2_euler_path" class="euler clip" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
</clipPath>
<mask id="intersection_mask_euler">
<rect fill="white" x="-100" y="-100" width="200" height="200"/>
<path id="intersection_mask1_euler_path" class="euler mask" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path id="intersection_mask2_euler_path" class="euler mask" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
</mask>

<rect class="euler_yay" clip-path="url('#intersection_clip2_euler')" mask="url('#intersection_mask_euler')" x="-100" y="-100" width="200" height="200"/>

<text x="0" y="-70">&#xe904;</text>
<text x="66.57" y="-21.6311">&#xe903;</text>
<text x="41.1449" y="56.6311">&#xe902;</text>
<text x="-41.1449" y="56.6311">&#xe901;</text>
<text x="-66.57" y="-21.6311">&#xe900;</text>
</svg>"###;

static EULER_SVG_SETMINUS: &str = r###"<svg version="1.1" viewBox="-100 -100 200 200" xmlns="http://www.w3.org/2000/svg" class="eulersvg">
<path class="cd1 bgclll1" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path class="cd3 bgclll3" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>

<clipPath id="setminus_clip1_euler">
<path id="setminus_clip1_euler_path" class="euler clip" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
</clipPath>
<mask id="setminus_mask_euler">
<rect fill="white" x="-100" y="-100" width="200" height="200"/>
<path id="setminus_mask1_euler_path" class="euler mask" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path id="setminus_mask2_euler_path" class="euler mask" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
<path id="setminus_clip2_euler_path" style="fill: black; stroke: black" class="euler clip" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
</mask>

<rect class="euler_yay" clip-path="url('#setminus_clip1_euler')" mask="url('#setminus_mask_euler')" x="-100" y="-100" width="200" height="200"/>

<text x="0" y="-70">&#xe904;</text>
<text x="66.57" y="-21.6311">&#xe903;</text>
<text x="41.1449" y="56.6311">&#xe902;</text>
<text x="-41.1449" y="56.6311">&#xe901;</text>
<text x="-66.57" y="-21.6311">&#xe900;</text>
</svg>"###;

static EULER_SVG_UNION: &str = r###"<svg version="1.1" viewBox="-100 -100 200 200" xmlns="http://www.w3.org/2000/svg" class="eulersvg">
<path class="cd1 bgclll1" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path class="cd3 bgclll3" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
<clipPath id="union_clip1_euler">
<path id="union_clip1_euler_path" class="euler clip" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path id="union_clip2_euler_path" class="euler clip" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
</clipPath>
<mask id="union_mask_euler">
<rect fill="white" x="-100" y="-100" width="200" height="200"/>
<path id="union_mask1_euler_path" class="euler mask" d="M -66.57395614066075 -39.131189606246295 A 17.5 17.5 0 0 0 -76.86019805577904 -7.473392204684714 L 30.858725745354857, 70.78898700780789 A 17.5 17.5 0 0 0 57.78845669563833 62.03898700780788 L 83.21744517582593, -16.223392204684764 A 17.5 17.5 0 0 0 66.57395614066074 -39.13118960624634 L -66.57395614066075, -39.131189606246295"></path>
<path id="union_mask2_euler_path" class="euler mask" d="M -49.961746444860204 44.49593469062212 A 15 15 0 0 0 -32.328188876086 68.76644452187054 L 75.39073492504784, -9.495934690622128 A 15 15 0 0 0 57.757177356273644 -33.76644452187055 L -49.961746444860204, 44.49593469062212"></path>
</mask>

<rect class="euler_yay" clip-path="url('#union_clip1_euler')" mask="url('#union_mask_euler')" x="-100" y="-100" width="200" height="200"/>

<text x="0" y="-70">&#xe904;</text>
<text x="66.57" y="-21.6311">&#xe903;</text>
<text x="41.1449" y="56.6311">&#xe902;</text>
<text x="-41.1449" y="56.6311">&#xe901;</text>
<text x="-66.57" y="-21.6311">&#xe900;</text>
</svg>"###;
