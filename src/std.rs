/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

extern crate std;

use crate::{HandlerResult, Opt, Opts, ParseControl, ParseResult};

impl<ID: 'static> Opts<ID> {
  /// Wrapper around `parse` that gathers arguments from the command line and prints errors to stderr.
  ///
  /// Requires std
  pub fn parse_env<'a>(&self, handler: impl FnMut(&ID, &Opt<ID>, &str, &str) -> HandlerResult<'a, ParseControl>
  ) -> ParseResult {
    let mut argv = std::env::args();
    let argv0 = argv.next().unwrap();
    let program_name = std::path::Path::new(&argv0).file_name().unwrap().to_string_lossy();
    self.parse(&program_name, argv, handler, |e| { std::eprintln!("error: {e}") })
  }
}
