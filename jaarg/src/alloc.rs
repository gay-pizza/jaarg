/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use crate::{Opts, ParseControl, ParseError, ParseResult};

impl Opts<&'static str> {
  /// Parse an iterator of strings as arguments and return the results in a [`BTreeMap`].
  ///
  /// Requires `features = ["alloc"]`.
  pub fn parse_map<'a, S: AsRef<str> + 'a, I: Iterator<Item = S>>(&self, program_name: &str, args: I,
    help: impl Fn(&str), error: impl FnOnce(&str, ParseError)
  ) -> ParseMapResult {
    let mut out: BTreeMap<&'static str, String> = BTreeMap::new();
    match self.parse(&program_name, args, |_program_name, id, opt, _name, arg| {
      if opt.is_help() {
        help(program_name);
        Ok(ParseControl::Quit)
      } else {
        out.insert(id, arg.into());
        Ok(ParseControl::Continue)
      }
    }, error) {
      ParseResult::ContinueSuccess => ParseMapResult::Map(out),
      ParseResult::ExitSuccess => ParseMapResult::ExitSuccess,
      ParseResult::ExitError => ParseMapResult::ExitFailure,
    }
  }
}

/// The result of parsing commands with [Opts::parse_map].
pub enum ParseMapResult {
  Map(BTreeMap<&'static str, String>),
  ExitSuccess, ExitFailure
}
