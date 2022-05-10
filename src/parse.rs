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
                    } else if macro_name == b"pretty_venn_duration" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"60s"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"lorem" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, LOREM), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$mid" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\mid"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol0" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"><span class="symbol0"></span></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol0" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\htmlClass{symbol0}{}}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol1" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"><span class="symbol1"></span></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol1" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\htmlClass{symbol1}{}}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol2" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"><span class="symbol2"></span></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol2" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\htmlClass{symbol2}{}}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol3" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"><span class="symbol3"></span></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol3" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\htmlClass{symbol3}{}}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"symbol4" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"<span class="symbol_container"><span class="symbol4"></span></span>"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$symbol4" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, r###"\htmlClass{symbol_container}{\htmlClass{symbol4}{}}"###), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"hr" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, "<hr>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"br" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, "<br>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$ldots" {
                        self.pm(|t, p, a| OutInternal::Const(t, p, a, "\\ldots"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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
                    } else if macro_name == b"verbatim" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="verbatim">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"video_container" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<div class="slightlywide"><div class="video_container">"###, "</div></div>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"solution" {
                        self.pm(|t, p, a| OutInternal::Toggled(t, p, a, r###"Show a possible solution"###, "Hide solution"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"proof_as_exercise" {
                        self.pm(|t, p, a| OutInternal::Toggled(t, p, a, r###"Prove it yourself, then click to compare"###, "Hide proof"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"nobr" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, r###"<span class="nobr">"###, "</span>"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$tag" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\tag{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$p" {
                        self.pm(OutInternal::MathGroupingParens, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$text" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\text{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{highlightmath highlightc1}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight2" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{highlightmath highlightc2}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight3" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{highlightmath highlightc3}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{highlightmathdirect highlightc1}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight2_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{highlightmathdirect highlightc2}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$highlight3_direct" {
                        self.pm(|t, p, a| OutInternal::Enclose(t, p, a, "\\htmlClass{highlightmathdirect highlightc3}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$class" {
                        self.pm(|t, p, a| OutInternal::Enclose2(t, p, a, "\\htmlClass{", "}{", "}"), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"link" {
                        self.pm(OutInternal::Link, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"captioned" {
                        self.pm(OutInternal::Captioned, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"set_math_id" {
                        self.pm(OutInternal::SetMathId, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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
                        self.pm(|t, p, a| OutInternal::EncloseMath(t, p, a, "twice".into(), r###"\operatorname{twice}("###.into(), r###")"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
                    } else if macro_name == b"$powerset" {
                        self.pm(|t, p, a| OutInternal::EncloseFunctionApplication(t, p, a, "powerset".into(), r###"\operatorname{\mathcal{P}}"###.into()), y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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
