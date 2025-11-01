/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

/// Static structure that contains instructions for parsing command-line arguments
pub struct Opts<ID: 'static> {
  /// List of options
  options: &'static[Opt<ID>],
  /// String containing single characters that match option prefixes
  flag_chars: &'static str,
  /// A description of what the program does
  description: Option<&'static str>,
}

impl<ID: 'static> Opts<ID> {
  /// Build argument parser options with the default flag character of '-'
  pub const fn new(options: &'static[Opt<ID>]) -> Self {
    Self {
      options,
      flag_chars: "-",
      description: None,
    }
  }

  /// Set the recognised flag/option characters.
  pub const fn with_flag_chars(mut self, flag_chars: &'static str) -> Self {
    self.flag_chars = flag_chars;
    self
  }

  /// Set the description of the program, available to help writers.
  pub const fn with_description(mut self, description: &'static str) -> Self {
    self.description = Some(description);
    self
  }
}
