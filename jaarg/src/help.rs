/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

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
        .filter(|o| matches!(o.r#type, OptType::Value | OptType::Flag)) {
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
        .filter(|o| matches!(o.r#type, OptType::Positional)) {
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
    use core::fmt::Write;

    // Base short usage
    writeln!(f, "{}", StandardShortUsageWriter::new(self.0.clone()))?;

    if let Some(description) = self.0.options.description {
      writeln!(f)?;
      writeln!(f, "{description}")?;
    }

    fn calculate_left_pad<ID: 'static>(option: &Opt<ID>) -> usize {
      (match option.names {
        OptIdentifier::Single(name) => name.chars().count(),
        OptIdentifier::Multi(names) => (names.len() - 1) * 3 + names.iter()
          .fold(0, |accum, name| accum + name.chars().count()),
      }) + option.value_name.map_or(0, |v| v.len() + 3)
    }

	  // Determine the alignment width from the longest option parameter
    let align_width = 2 + self.0.options.iter()
      .map(|o| calculate_left_pad(o)).max().unwrap_or(0);

    // Write positional argument descriptions
    let mut first = true;
    for option in self.0.options.iter()
        .filter(|o| matches!(o.r#type, OptType::Positional)) {
      if first {
        // Write separator and positional section header
        writeln!(f)?;
        writeln!(f, "Positional arguments:")?;
        first = false;
      }

      // Write positional argument line
      write!(f, "  {name}", name = option.first_name())?;
      if let Some(help_text) = option.help_string {
        write!(f, " {:.<width$} {help_text}", "",
          width = align_width - calculate_left_pad(option))?;
      }
      writeln!(f)?;
    }

    // Write option parameter argument descriptions
    first = true;
    for option in self.0.options.iter()
        .filter(|o| matches!(o.r#type, OptType::Flag | OptType::Value)) {
      if first {
        // Write separator and options section header
        writeln!(f)?;
        writeln!(f, "Options:")?;
        first = false;
      }

      // Write option flag name(s)
      match option.names {
        OptIdentifier::Single(name) => {
          write!(f, "  {name}")?;
        }
        OptIdentifier::Multi(names) => for (i, name) in names.iter().enumerate() {
          write!(f, "{prefix}{name}", prefix = if i == 0 { "  " } else { " | " })?;
        }
      }

      // Write value argument for value options parameters
      if let Some(value_name) = option.value_name {
        write!(f, " <{value_name}>")?;
      }

      // Write padding and help text
      if let Some(help_text) = option.help_string {
        write!(f, " {:.<width$} {help_text}", "",
          width = align_width - calculate_left_pad(option))?;
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
