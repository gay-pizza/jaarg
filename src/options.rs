/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

/// Static structure that contains instructions for parsing command-line arguments
pub struct Opts<ID: 'static> {
  /// String containing single characters that match option prefixes
  flag_chars: &'static str,
  /// List of options
  options: &'static[Opt<ID>],
}

impl<ID: 'static> Opts<ID> {
  /// Build argument parser options with the default flag character of '-'
  pub const fn new(options: &'static[Opt<ID>]) -> Self {
    Self { flag_chars: "-", options }
  }
  pub const fn new_flag(flag_chars: &'static str, options: &'static[Opt<ID>]) -> Self {
    Self { flag_chars, options }
  }
}
