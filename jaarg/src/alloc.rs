/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

extern crate alloc;

use crate::{Opts, ParseControl, ParseError, ParseResult};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};

impl Opts<&'static str> {
  /// Parse an iterator of strings as arguments and return the results in a [`BTreeMap`].
  ///
  /// Requires `features = ["alloc"]`.
  pub fn parse_map<'opt, 't, S: AsRef<str> + 't, I: Iterator<Item = S>>(&'opt self, program_name: &str, args: I,
    help: impl Fn(&str), error: impl FnOnce(&str, ParseError)
  ) -> ParseMapResult {
    let mut out: BTreeMap<&'static str, String> = BTreeMap::new();
    match self.parse(program_name, args, |ctx| {
      if ctx.option.is_help() {
        help(program_name);
        Ok(ParseControl::Quit)
      } else {
        out.insert(ctx.id, ctx.arg.map_or(String::new(), |o| o.to_string()));
        Ok(ParseControl::Continue)
      }
    }, error) {
      ParseResult::ContinueSuccess => ParseMapResult::Map(out),
      ParseResult::ExitSuccess => ParseMapResult::ExitSuccess,
      ParseResult::ExitFailure => ParseMapResult::ExitFailure,
    }
  }
}

/// The result of parsing commands with [Opts::parse_map].
pub enum ParseMapResult {
  Map(BTreeMap<&'static str, String>),
  ExitSuccess, ExitFailure
}
