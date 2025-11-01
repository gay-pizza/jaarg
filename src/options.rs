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

type BitSetType = u32;
const BITSET_SLOTS: usize = 4;
/// The maximum amount of allowed required non-positional options.
pub const MAX_REQUIRED_OPTIONS: usize = BitSetType::BITS as usize * BITSET_SLOTS;

impl<ID: 'static> Opts<ID> {
  /// Build argument parser options with the default flag character of '-'
  pub const fn new(options: &'static[Opt<ID>]) -> Self {
    // Validate passed options
    let mut opt_idx = 0;
    let mut num_required_parameters = 0;
    while opt_idx < options.len() {
      if matches!(options[opt_idx].r#type, OptType::Flag | OptType::Value) && options[opt_idx].is_required() {
        num_required_parameters += 1;
      }
      opt_idx += 1;
    }
    assert!(num_required_parameters <= MAX_REQUIRED_OPTIONS,
      "More than 128 non-positional required option entries is not supported at this time");

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
