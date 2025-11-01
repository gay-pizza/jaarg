/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

extern crate std;

use std::{env, eprintln, println};
use std::path::Path;
use crate::{HandlerResult, Opt, Opts, ParseControl, ParseResult, StandardShortUsageWriter, HelpWriterContext, StandardFullHelpWriter, HelpWriter};

impl<ID: 'static> Opts<ID> {
  /// Wrapper around `jaarg::parse` that gathers arguments from the command line and prints errors to stderr.
  /// The errors are formatted in a standard user-friendly format.
  ///
  /// Requires features = [std]
  pub fn parse_easy<'a>(&self, handler: impl FnMut(&str, &ID, &Opt<ID>, &str, &str) -> HandlerResult<'a, ParseControl>
  ) -> ParseResult {
    let mut argv = env::args();
    let argv0 = argv.next().unwrap();
    let program_name = Path::new(&argv0).file_name().unwrap().to_string_lossy();
    self.parse(&program_name, argv, handler, |program_name, e| {
      eprintln!("{program_name}: {e}");
      self.eprint_help::<StandardShortUsageWriter<'_, ID>>(program_name);
      eprintln!("Run '{program_name} --help' to view all available options.");
    })
  }

  /// Prints full help text for the options using the standard full
  ///
  /// Requires features = [std]
  pub fn print_full_help(&self, program_name: &str) {
    self.print_help::<StandardFullHelpWriter<'_, ID>>(program_name);
  }

  /// Print help text to stdout using the provided help writer
  ///
  /// Requires features = [std]
  pub fn print_help<'a, W: HelpWriter<'a, ID>>(&'a self, program_name: &'a str) {
    let ctx = HelpWriterContext { options: self, program_name };
    println!("{}", W::new(ctx));
  }

  /// Print help text to stderr using the provided help writer
  ///
  /// Requires features = [std]
  pub fn eprint_help<'a, W: HelpWriter<'a, ID>>(&'a self, program_name: &'a str) {
    let ctx = HelpWriterContext { options: self, program_name };
    eprintln!("{}", W::new(ctx));
  }
}
