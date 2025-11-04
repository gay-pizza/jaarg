/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

/// Static structure that contains instructions for parsing command-line arguments.
pub struct Opts<ID: 'static> {
  /// List of options
  options: &'static[Opt<ID>],
  /// String containing single characters that match option prefixes
  flag_chars: &'static str,
  /// A description of what the program does
  description: Option<&'static str>,
}

type RequiredParamsBitSet = ordered_bitset::OrderedBitSet<u32, 4>;

/// The maximum amount of allowed required non-positional options.
pub const MAX_REQUIRED_OPTIONS: usize = RequiredParamsBitSet::CAPACITY;

impl<ID: 'static> Opts<ID> {
  /// Build argument parser options with the default flag character of '-'.
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
    assert!(num_required_parameters <= RequiredParamsBitSet::CAPACITY,
      "More than 128 non-positional required option entries is not supported at this time");

    Self {
      options,
      flag_chars: "-",
      description: None,
    }
  }

  /// Sets the recognised flag/option characters.
  #[inline]
  pub const fn with_flag_chars(mut self, flag_chars: &'static str) -> Self {
    self.flag_chars = flag_chars;
    self
  }

  /// Sets the description of the program, available to help writers.
  #[inline]
  pub const fn with_description(mut self, description: &'static str) -> Self {
    self.description = Some(description);
    self
  }

  /// Gets the first available help option if one exists.
  pub const fn help_option(&self) -> Option<&'static Opt<ID>> {
    let mut i = 0;
    while i < self.options.len() {
      if self.options[i].is_help() {
        return Some(&self.options[i]);
      }
      i += 1;
    }
    None
  }

  /// Gets an iterator over the parser's options.
  #[inline]
  pub fn iter(&self) -> core::slice::Iter<'static, Opt<ID>> {
    self.options.iter()
  }
}
