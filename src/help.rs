/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

pub struct HelpWriterContext<'a, ID: 'static> {
  pub options: &'a Opts<ID>,
  pub program_name: &'a str,
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
    for option in self.0.options.options.iter()
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
    for option in self.0.options.options.iter()
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

    // Base usage
    write!(f, "Usage: {}", self.0.program_name)?;
    let short_flag = self.0.options.flag_chars.chars().next().unwrap();

    // Write optional short options
    let mut first = true;
    for option in self.0.options.options {
      if let (OptType::Flag | OptType::Value, false) = (option.r#type, option.is_required()) {
        if let Some(c) = option.first_short_name_char() {
          if first {
            write!(f, " [{short_flag}")?;
            first = false;
          }
          f.write_char(c)?;
        }
      }
    }
    if !first {
      f.write_char(']')?;
    }

    // Write required short options
    first = true;
    for option in self.0.options.options {
      if let (OptType::Flag | OptType::Value, true) = (option.r#type, option.is_required()) {
        if let Some(c) = option.first_short_name_char() {
          if first {
            write!(f, " <{short_flag}")?;
            first = false;
          }
          f.write_char(c)?;
        }
      }
    }
    if !first {
      f.write_char('>')?;
    }

    // Write positional arguments
    for option in self.0.options.options.iter()
        .filter(|o| matches!(o.r#type, OptType::Positional)) {
      let name = option.first_name();
      match option.is_required() {
        true  => write!(f, " <{name}>")?,
        false => write!(f, " [{name}]")?,
      }
    }
    writeln!(f)?;

    fn calculate_left_pad<ID: 'static>(option: &Opt<ID>) -> usize {
      (match option.names {
        OptIdentifier::Single(name) => name.chars().count(),
        OptIdentifier::Multi(names) => (names.len() - 1) * 3 + names.iter()
          .fold(0, |accum, name| accum + name.chars().count()),
      }) + option.value_name.map_or(0, |v| v.len() + 3)
    }

	  // Determine the alignment width from the longest option parameter
    let align_width = 2 + self.0.options.options.iter()
      .map(|o| calculate_left_pad(o)).max().unwrap_or(0);

    // Write positional argument descriptions
    first = true;
    for option in self.0.options.options.iter()
        .filter(|o| matches!(o.r#type, OptType::Positional)) {
      if first {
        // Write separator and positional section header
        writeln!(f)?;
        writeln!(f, "Positional arguments:")?;
        first = false;
      }

      // Write positional argument line
      writeln!(f, "  {name} {:.<width$} {help_text}", "",
        name = option.first_name(),
        help_text = option.help_string,
        width = align_width - calculate_left_pad(option))?;
    }

    // Write option parameter argument descriptions
    first = true;
    for option in self.0.options.options.iter()
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
      writeln!(f, " {:.<width$} {help_text}", "",
        help_text = option.help_string,
        width = align_width - calculate_left_pad(option))?;
    }

    Ok(())
  }
}
