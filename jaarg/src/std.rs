/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

extern crate std;

use crate::{
  alloc::ParseMapResult, ErrorUsageWriter, ErrorUsageWriterContext, HandlerResult, HelpWriter, HelpWriterContext,
  Opts, ParseControl, ParseError, ParseHandlerContext, ParseResult, StandardErrorUsageWriter, StandardFullHelpWriter
};
use std::path::Path;
use std::rc::Rc;
use std::{env, eprint, print};

impl<ID: 'static> Opts<ID> {
  /// Wrapper around [Opts::parse] that gathers arguments from the command line and prints errors to stderr.
  /// The errors are formatted in a standard user-friendly format.
  ///
  /// Requires `features = ["std"]`.
  pub fn parse_easy<'a>(&self, handler: impl FnMut(ParseHandlerContext<ID>) -> HandlerResult<'a, ParseControl>
  ) -> ParseResult {
    let (program_name, argv) = Self::easy_args();
    self.parse(&program_name, argv, handler,
      |name, e| self.eprint_usage::<StandardErrorUsageWriter<'_, ID>>(name, e))
  }

  /// Prints full help text for the options using the standard full.
  ///
  /// Requires `features = ["std"]`.
  pub fn print_full_help(&self, program_name: &str) {
    self.print_help::<StandardFullHelpWriter<'_, ID>>(program_name);
  }

  /// Print help text to stdout using the provided help writer.
  ///
  /// Requires `features = ["std"]`.
  pub fn print_help<'a, W: HelpWriter<'a, ID>>(&'a self, program_name: &'a str) {
    let ctx = HelpWriterContext { options: self, program_name };
    print!("{}", W::new(ctx));
  }

  /// Print help text to stderr using the provided help writer.
  ///
  /// Requires `features = ["std"]`.
  pub fn eprint_help<'a, W: HelpWriter<'a, ID>>(&'a self, program_name: &'a str) {
    let ctx = HelpWriterContext { options: self, program_name };
    eprint!("{}", W::new(ctx));
  }

  /// Print error & usage text to stderr using the provided error & usage writer.
  ///
  /// Requires `features = ["std"]`.
  pub fn eprint_usage<'a, W: ErrorUsageWriter<'a, ID>>(&'a self, program_name: &'a str, error: ParseError<'a>) {
    let ctx = ErrorUsageWriterContext { options: self, program_name, error };
    eprint!("{}", W::new(ctx));
  }

  fn easy_args() -> (Rc<str>, env::Args) {
    let mut argv = env::args();
    let argv0 = argv.next().unwrap();
    let program_name = Path::new(&argv0).file_name().unwrap().to_string_lossy();
    (program_name.into(), argv)
  }
}

impl Opts<&'static str> {
  /// Parse arguments from the command line and return the results in a [`alloc::collections::BTreeMap`].
  /// Help and errors are formatted in a standard user-friendly format.
  ///
  /// Requires `features = ["std"]`.
  pub fn parse_map_easy(&self) -> ParseMapResult {
    let (program_name, argv) = Self::easy_args();
    self.parse_map(&program_name, argv,
      |name| self.print_full_help(name),
      |name, e| self.eprint_usage::<StandardErrorUsageWriter<'_, &'static str>>(name, e))
  }
}
