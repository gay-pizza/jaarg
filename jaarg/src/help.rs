/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use crate::{Opt, Opts, ParseError};
use crate::option::{OptIdentifier, OptType};

/// Enough context to show full help text.
pub struct HelpWriterContext<'a, ID: 'static> {
  pub options: &'a Opts<ID>,
  pub program_name: &'a str,
}

impl<ID: 'static> Clone for HelpWriterContext<'_, ID> {
  fn clone(&self) -> Self {
    Self { options: self.options, program_name: self.program_name }
  }
}

pub trait HelpWriter<'a, ID: 'static>: core::fmt::Display {
  fn new(ctx: HelpWriterContext<'a, ID>) -> Self;
}

pub struct StandardShortUsageWriter<'a, ID: 'static>(HelpWriterContext<'a, ID>);

impl<'a, ID: 'static> HelpWriter<'a, ID> for StandardShortUsageWriter<'a, ID> {
  fn new(ctx: HelpWriterContext<'a, ID>) -> Self { Self(ctx) }
}

impl<ID: 'static> core::fmt::Display for StandardShortUsageWriter<'_, ID> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Usage: {}", self.0.program_name)?;

    // Write option parameter arguments
    for option in self.0.options.iter()
        .filter(|o| matches!((o.r#type, o.is_short_visible()), (OptType::Value | OptType::Flag, true))) {
      write!(f, " {}", if option.is_required() { '<' } else { '[' })?;
      match (option.first_short_name(), option.first_long_name()) {
        (Some(short_name), Some(long_name)) => write!(f, "{short_name}|{long_name}")?,
        (Some(short_name), None) => f.write_str(short_name)?,
        (None, Some(long_name))  => f.write_str(long_name)?,
        _ => unreachable!(),
      }
      if let Some(value_name) = option.value_name {
         write!(f, " {value_name}")?;
      }
      write!(f, "{}", if option.is_required() { '>' } else { ']' })?;
    }

    // Write positional arguments
    for option in self.0.options.iter()
        .filter(|o| matches!((o.r#type, o.is_short_visible()), (OptType::Positional, true))) {
      let name = option.first_name();
      match option.is_required() {
        true  => write!(f, " <{name}>")?,
        false => write!(f, " [{name}]")?,
      }
    }
    Ok(())
  }
}

pub struct StandardFullHelpWriter<'a, ID: 'static>(HelpWriterContext<'a, ID>);

impl<'a, ID: 'static> HelpWriter<'a, ID> for StandardFullHelpWriter<'a, ID> {
  fn new(ctx: HelpWriterContext<'a, ID>) -> Self { Self(ctx) }
}

impl<ID> core::fmt::Display for StandardFullHelpWriter<'_, ID> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    // Base short usage
    writeln!(f, "{}", StandardShortUsageWriter::new(self.0.clone()))?;

    if let Some(description) = self.0.options.description {
      writeln!(f)?;
      writeln!(f, "{description}")?;
    }

	  // Determine the alignment width from the longest option parameter
    fn calculate_option_line_length<ID: 'static>(option: &Opt<ID>) -> usize {
      (match option.names {
        OptIdentifier::Single(name) => name.chars().count(),
        OptIdentifier::Multi(names) => (names.len() - 1) * 3 + names.iter()
          .fold(0, |accum, name| accum + name.chars().count()),
      }) + option.value_name.map_or(0, |v| v.len() + 3)
    }
    let align_width = 3 + self.0.options.iter()
      .map(|o| calculate_option_line_length(o)).max().unwrap_or(0);

    // Write positional argument descriptions
    let mut first = true;
    for option in self.0.options.iter()
        .filter(|o| matches!((o.r#type, o.is_full_visible()), (OptType::Positional, true))) {
      if first {
        // Write separator and positional section header
        writeln!(f)?;
        writeln!(f, "Positional arguments:")?;
        first = false;
      }

      // Write positional argument line (name + optional aligned help text)
      let name = option.first_name();
      write!(f, "  {name}")?;
      if let Some(help_text) = option.help_string {
        write!(f, " {:.<width$} {help_text}", "",
          width = align_width - name.chars().count() - 1)?;
      }
      writeln!(f)?;
    }

    /// Formatter for option usage lines.
    struct OptionUsageLine<'a, ID>(&'a Opt<ID>);
    impl<ID> core::fmt::Display for OptionUsageLine<'_, ID> {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;
        let mut length = 0;

        // Write option flag name(s)
        match self.0.names {
          OptIdentifier::Single(name) => {
            write!(f, "{name}")?;
            length = name.chars().count();
          }
          OptIdentifier::Multi(names) => {
            for (i, name) in names.iter().enumerate() {
              if i == 0 {
                write!(f, "{name}")?;
                length += name.chars().count();
              } else {
                write!(f, " | {name}")?;
                length += 3 + name.chars().count();
              }
            }
          }
        }

        // Write value argument for value options parameters
        if let Some(value_name) = self.0.value_name {
          write!(f, " <{value_name}>")?;
          length += 2 + value_name.chars().count() + 1;
        }

        // Write padding if requested
        match (f.align(), f.width().unwrap_or(0).checked_sub(length)) {
          (Some(core::fmt::Alignment::Left), Some(width)) if width > 0 => {
            let fill = f.fill();
            // First padding char is *always* a space
            f.write_char(' ')?;
            for _ in 1..width {
              f.write_char(fill)?;
            }
            Ok(())
          }
          _ => Ok(()),
        }
      }
    }

    // Write option parameter argument descriptions
    first = true;
    for option in self.0.options.iter()
        .filter(|o| matches!((o.r#type, o.is_full_visible()), (OptType::Flag | OptType::Value, true))) {
      if first {
        // Write separator and options section header
        writeln!(f)?;
        writeln!(f, "Options:")?;
        first = false;
      }

      // Write line for option, with aligned help text if needed
      let line = OptionUsageLine(option);
      if let Some(help_text) = option.help_string {
        write!(f, "  {line:.<align_width$} {help_text}")?;
      } else {
        write!(f, "  {line}")?;
      }
      writeln!(f)?;
    }

    Ok(())
  }
}


// Enough context to show usage and error information.
pub struct ErrorUsageWriterContext<'a, ID: 'static> {
  pub options: &'a Opts<ID>,
  pub program_name: &'a str,
  pub error: ParseError<'a>
}

pub trait ErrorUsageWriter<'a, ID: 'static>: core::fmt::Display {
  fn new(ctx: ErrorUsageWriterContext<'a, ID>) -> Self;
}

pub struct StandardErrorUsageWriter<'a, ID: 'static>(ErrorUsageWriterContext<'a, ID>);

impl<'a, ID: 'static> ErrorUsageWriter<'a, ID> for StandardErrorUsageWriter<'a, ID> {
  fn new(ctx: ErrorUsageWriterContext<'a, ID>) -> Self { Self(ctx) }
}

impl<ID> core::fmt::Display for StandardErrorUsageWriter<'_, ID> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    // Write error
    writeln!(f, "{name}: {error}", name=self.0.program_name, error = self.0.error)?;

    // Provide usage hint for missing required arguments
    if matches!(self.0.error, ParseError::RequiredPositional(_) | ParseError::RequiredParameter(_)) {
      // Write short usage
      writeln!(f, "{}", StandardShortUsageWriter::new(HelpWriterContext {
        options: self.0.options,
        program_name: self.0.program_name
      }))?;

      // Write full help instruction if available
      if let Some(help_option) = self.0.options.help_option() {
        writeln!(f, "Run '{name} {help}' to view all available options.",
          name = self.0.program_name,
          // Prefer long name, but otherwise any name is fine
          help = help_option.first_long_name().unwrap_or(help_option.first_name()))?;
      }
    }
    Ok(())
  }
}
