/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

extern crate std;

use crate::{HandlerResult, HelpWriter, HelpWriterContext, Opt, Opts, ParseControl, ParseError, ParseResult, StandardFullHelpWriter, StandardShortUsageWriter};
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::string::String;
use std::{env, eprintln, println};

impl<ID: 'static> Opts<ID> {
  /// Wrapper around [Opts::parse] that gathers arguments from the command line and prints errors to stderr.
  /// The errors are formatted in a standard user-friendly format.
  ///
  /// Requires `features = [std]`.
  pub fn parse_easy<'a>(&self, handler: impl FnMut(&str, &ID, &Opt<ID>, &str, &str) -> HandlerResult<'a, ParseControl>
  ) -> ParseResult {
    let (program_name, argv) = Self::easy_args();
    self.parse(&program_name, argv, handler, |name, e| self.easy_error(name, e))
  }

  /// Prints full help text for the options using the standard full.
  ///
  /// Requires `features = [std]`.
  pub fn print_full_help(&self, program_name: &str) {
    self.print_help::<StandardFullHelpWriter<'_, ID>>(program_name);
  }

  /// Print help text to stdout using the provided help writer.
  ///
  /// Requires `features = [std]`.
  pub fn print_help<'a, W: HelpWriter<'a, ID>>(&'a self, program_name: &'a str) {
    let ctx = HelpWriterContext { options: self, program_name };
    println!("{}", W::new(ctx));
  }

  /// Print help text to stderr using the provided help writer.
  ///
  /// Requires `features = [std]`.
  pub fn eprint_help<'a, W: HelpWriter<'a, ID>>(&'a self, program_name: &'a str) {
    let ctx = HelpWriterContext { options: self, program_name };
    eprintln!("{}", W::new(ctx));
  }

  fn easy_args<'a>() -> (Rc<str>, env::Args) {
    let mut argv = env::args();
    let argv0 = argv.next().unwrap();
    let program_name = Path::new(&argv0).file_name().unwrap().to_string_lossy();
    (program_name.into(), argv)
  }

  fn easy_error(&self, program_name: &str, err: ParseError) {
    eprintln!("{program_name}: {err}");
    self.eprint_help::<StandardShortUsageWriter<'_, ID>>(program_name);
    if let Some(help_option) = self.help_option() {
      eprintln!("Run '{program_name} {help}' to view all available options.",
        help = help_option.first_long_name().unwrap_or(help_option.first_name()));
    }
  }
}

/// The result of parsing commands with [Opts::parse_map].
pub enum ParseMapResult {
  Map(BTreeMap<&'static str, String>),
  Exit(std::process::ExitCode),
}

impl Opts<&'static str> {
  /// Parse an iterator of strings as arguments and return the results in a [BTreeMap].
  ///
  /// Requires `features = [std]`.
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
      ParseResult::ExitSuccess => ParseMapResult::Exit(std::process::ExitCode::SUCCESS),
      ParseResult::ExitError => ParseMapResult::Exit(std::process::ExitCode::FAILURE),
    }
  }

  /// Parse arguments from the command line and return the results in a [BTreeMap].
  /// Help and errors are formatted in a standard user-friendly format.
  ///
  /// Requires `features = [std]`.
  pub fn parse_map_easy(&self) -> ParseMapResult {
    let (program_name, argv) = Self::easy_args();
    self.parse_map(&program_name, argv,
      |name| self.print_full_help(name),
      |name, e| self.easy_error(name, e))
  }
}
