use thiserror::Error;
use ropey::Rope;
use serde::Deserialize;

use atm_parser_helper::{Eoi, Error, ParserHelper};
use valuable_value::human::{VVDeserializer, Error as VVError};

use crate::{Yatt, RunConfiguration};
use crate::macros::{OutInternal, Trace};

// An offset into the source map.
pub(crate) type Offset = usize;

pub(crate) type OffsetSpan = (Offset /* start, inclusive */, Offset /* end, exclusive */);

#[derive(Error, Debug)]
pub(crate) enum ParseError {
    #[error("unexpected end of input")]
    Eoi,
    #[error("TODO")]
    Parameters(VVError, Trace),
    #[error("TODO")]
    UnknownMacroName(Trace),
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
                    let trace_start = source_offset + self.p.position() - (initial_position + 1);
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
                    } else if macro_name == b"span" {
                        self.pm(OutInternal::Span, y, source_offset, parse_parameters, initial_position, trace_start, &mut outs, &mut start, &mut last_non_ws)?;
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

    fn parse_args(&mut self, y: &mut Yatt, source_offset: Offset) -> Result<Box<[OutInternal]>, ParseError> {
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
        M: Fn(Trace, P, Box<[OutInternal]>) -> OutInternal,
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

// pub(crate) enum Ast {
//     // Text that will be written somewhere without needing any processing beyond handling escape sequences and ignoring comments.
//     Text(OffsetSpan),
//     // Invocation of a plugin-defined macro.
//     Invocation {
//         name: OffsetSpan,
//         m: Macro,
//         // options: OffsetSpan,
//         arguments: Vec<Ast>,
//     },
//     // // Invocation of the built-in `input` macro.
//     // Input {
//     //     name: OffsetSpan,
//     //     argument: OffsetSpan,
//     //     resolved: Vec<Ast>,
//     // },
//     // // Invocation of the built-in `output` macro.
//     // Output {
//     //     name: OffsetSpan,
//     //     path: OffsetSpan,
//     //     contents: Vec<Ast>,
//     // },
// }
