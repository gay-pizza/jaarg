/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#[derive(Debug, Copy, Clone, PartialEq)]
enum OptType {
  Positional,
  Flag,
  Value,
}

#[derive(Debug, PartialEq)]
enum OptIdentifier {
  Single(&'static str),
  Multi(&'static[&'static str]),
}

/// Represents an option argument or positional argument to be parsed.
#[derive(Debug, PartialEq)]
pub struct Opt<ID> {
  id: ID,
  names: OptIdentifier,
  value_name: Option<&'static str>,
  help_string: Option<&'static str>,
  r#type: OptType,
  flags: OptFlag,
}

pub enum OptHide {
  Short,
  Full,
  All,
}

#[derive(Debug, PartialEq)]
struct OptFlag(u8);

impl OptFlag {
  #[allow(dead_code)]
  pub const NONE: Self          = Self(0);
  pub const REQUIRED: Self      = OptFlag(1 << 0);
  pub const HELP: Self          = OptFlag(1 << 1);
  pub const VISIBLE_SHORT: Self = OptFlag(1 << 2);
  pub const VISIBLE_FULL: Self  = OptFlag(1 << 3);

  pub const DEFAULT: Self = Self(Self::VISIBLE_SHORT.0 | Self::VISIBLE_FULL.0);
}

// TODO: Improve this interface by making the name field take AsOptIdentifier when const traits are stabilised
impl<ID> Opt<ID> {
  #[inline]
  const fn new(id: ID, names: OptIdentifier, value_name: Option<&'static str>, r#type: OptType) -> Self {
    assert!(match names {
      OptIdentifier::Single(_) => true,
      OptIdentifier::Multi(names) => !names.is_empty(),
    }, "Option names cannot be an empty slice");
    Self { id, names, value_name, help_string: None, r#type, flags: OptFlag::DEFAULT }
  }

  /// A positional argument that is parsed sequentially without being invoked by an option flag.
  pub const fn positional(id: ID, name: &'static str) -> Self {
    Self::new(id, OptIdentifier::Single(name), None, OptType::Positional)
  }
  /// A flag-type option that serves as the interface's help flag.
  pub const fn help_flag(id: ID, names: &'static[&'static str]) -> Self {
    Self::new(id, OptIdentifier::Multi(names), None, OptType::Flag)
      .with_help_flag()
  }
  /// A flag-type option, takes no value.
  pub const fn flag(id: ID, names: &'static[&'static str]) -> Self {
    Self::new(id, OptIdentifier::Multi(names), None, OptType::Flag)
  }
  /// An option argument that takes a value.
  pub const fn value(id: ID, names: &'static[&'static str], value_name: &'static str) -> Self {
    Self::new(id, OptIdentifier::Multi(names), Some(value_name), OptType::Value)
  }

  /// This option is required, ie; parsing will fail if it is not specified.
  #[inline]
  pub const fn required(mut self) -> Self {
    assert!(!self.is_help(), "Help flag cannot be made required");
    self.flags.0 |= OptFlag::REQUIRED.0;
    self
  }

  /// Sets the help string for an option.
  #[inline]
  pub const fn help_text(mut self, help_string: &'static str) -> Self {
    self.help_string = Some(help_string);
    self
  }

  /// Marks the option to exclude it from appearing in short usage text, full help text, or both.
  #[inline]
  pub const fn hide_usage(mut self, from: OptHide) -> Self {
    self.flags.0 &= !match from {
      OptHide::Short => OptFlag::VISIBLE_SHORT.0,
      OptHide::Full  => OptFlag::VISIBLE_FULL.0,
      OptHide::All   => OptFlag::VISIBLE_SHORT.0 | OptFlag::VISIBLE_FULL.0,
    };
    self
  }

  #[inline]
  const fn with_help_flag(mut self) -> Self {
    assert!(matches!(self.r#type, OptType::Flag), "Only flags are allowed to be help options");
    self.flags.0 |= OptFlag::HELP.0;
    self
  }

  /// Returns true if this is a required positional argument, or required option argument.
  #[inline(always)]
  pub const fn is_required(&self) -> bool {
    (self.flags.0 & OptFlag::REQUIRED.0) != 0
  }

  /// Returns true if this is the help option.
  #[inline(always)]
  pub const fn is_help(&self) -> bool {
    (self.flags.0 & OptFlag::HELP.0) != 0
  }

  #[inline(always)]
  const fn is_short_visible(&self) -> bool {
    (self.flags.0 & OptFlag::VISIBLE_SHORT.0) != 0
  }

  #[inline(always)]
  const fn is_full_visible(&self) -> bool {
    (self.flags.0 & OptFlag::VISIBLE_FULL.0) != 0
  }
}

impl<ID: 'static> Opt<ID> {
  /// Get the first name of the option.
  pub const fn first_name(&self) -> &str {
    match self.names {
      OptIdentifier::Single(name) => name,
      OptIdentifier::Multi(names) => names.first().unwrap(),
    }
  }

  /// Get the first long option name, if one exists.
  pub const fn first_long_name(&self) -> Option<&'static str> {
    match self.names {
      OptIdentifier::Single(name) => if name.len() >= 3 { Some(name) } else { None },
      // Can be replaced with `find_map` once iterators are const fn
      OptIdentifier::Multi(names) => {
        let mut i = 0;
        while i < names.len() {
          if const_utf8::CharIterator::from(names[i]).count() >= 3 {
            return Some(names[i]);
          }
          i += 1;
        }
        None
      }
    }
  }

  /// Get the first short option name, if one exists.
  const fn first_short_name(&self) -> Option<&'static str> {
    const fn predicate(name: &str) -> bool {
      let mut chars = const_utf8::CharIterator::from(name);
      if let Some(first) = chars.next() {
        if let Some(c) = chars.next() {
          if c != first && chars.next().is_none() {
            return true
          }
        }
      }
      false
    }
    match self.names {
      OptIdentifier::Single(name) => if predicate(&name) { Some(name) } else { None },
      // Can be replaced with `find_map` once iterators are const fn
      OptIdentifier::Multi(names) => {
        let mut i = 0;
        while i < names.len() {
          if predicate(names[i]) {
            return Some(names[i]);
          }
          i += 1;
        }
        None
      }
    }
  }

  /// Get the first applicable short option's flag character, if one exists.
  const fn first_short_name_char(&self) -> Option<char> {
    const fn predicate(name: &str) -> Option<char> {
      let mut chars = const_utf8::CharIterator::from(name);
      if let Some(first) = chars.next() {
        if let Some(c) = chars.next() {
          if c != first && chars.next().is_none() {
            return Some(c)
          }
        }
      }
      None
    }
    match self.names {
      OptIdentifier::Single(name) => predicate(&name),
      // Can be replaced with `find_map` once iterators are const fn.
      OptIdentifier::Multi(names) => {
        let mut i = 0;
        while i < names.len() {
          if let Some(c) = predicate(names[i]) {
            return Some(c);
          }
          i += 1;
        }
        None
      }
    }
  }

  /// Search for a matching name in the option, offset allows to skip the first `n = offset` characters in the comparison.
  fn match_name(&self, string: &str, offset: usize) -> Option<&'static str> {
    let rhs = &string[offset..];
    if rhs.is_empty() {
      return None;
    }
    match self.names {
      OptIdentifier::Single(name) =>
        if &name[offset..] == rhs { Some(name) } else { None },
      OptIdentifier::Multi(names) =>
        names.iter().find(|name| &name[offset..] == rhs).map(|v| &**v),
    }
  }
}

impl core::ops::BitOr for OptFlag {
  type Output = Self;
  fn bitor(self, rhs: Self) -> Self::Output { Self(self.0 | rhs.0) }
}

#[cfg(test)]
mod opt_tests {
  use super::*;

  #[test]
  #[should_panic(expected = "Option names cannot be an empty slice")]
  fn test_new_empty_names_disallowed() {
    Opt::new((), OptIdentifier::Multi(&[]), None, OptType::Positional);
  }

  #[test]
  fn test_public_initialisers() {
    assert_eq!(Opt::positional((), "name"), Opt { id: (),
      names: OptIdentifier::Single("name"), value_name: None, help_string: None,
      r#type: OptType::Positional, flags: OptFlag::DEFAULT,
    });
    assert_eq!(Opt::help_flag((), &["name"]), Opt { id: (),
      names: OptIdentifier::Multi(&["name"]), value_name: None, help_string: None,
      r#type: OptType::Flag, flags: OptFlag::DEFAULT | OptFlag::HELP,
    });
    assert_eq!(Opt::flag((), &["name"]), Opt { id: (),
      names: OptIdentifier::Multi(&["name"]), value_name: None, help_string: None,
      r#type: OptType::Flag, flags: OptFlag::DEFAULT,
    });
    assert_eq!(Opt::value((), &["name"], "value"), Opt { id: (),
      names: OptIdentifier::Multi(&["name"]), value_name: Some("value"), help_string: None,
      r#type: OptType::Value, flags: OptFlag::DEFAULT,
    });
  }

  #[test]
  fn test_valid_with_chains() {
    assert_eq!(Opt::positional((), "").required(), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: None,
      r#type: OptType::Positional, flags: OptFlag::DEFAULT | OptFlag::REQUIRED,
    });
    assert_eq!(Opt::positional((), "").required().help_text("help string"), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: Some("help string"),
      r#type: OptType::Positional, flags: OptFlag::DEFAULT | OptFlag::REQUIRED,
    });
    assert_eq!(Opt::positional((), "").help_text("help string"), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: Some("help string"),
      r#type: OptType::Positional, flags: OptFlag::DEFAULT,
    });
    assert_eq!(Opt::positional((), "").hide_usage(OptHide::Short), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: None,
      r#type: OptType::Positional, flags: OptFlag::VISIBLE_FULL,
    });
    assert_eq!(Opt::positional((), "").hide_usage(OptHide::Full), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: None,
      r#type: OptType::Positional, flags: OptFlag::VISIBLE_SHORT,
    });
    assert_eq!(Opt::positional((), "").hide_usage(OptHide::All), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: None,
      r#type: OptType::Positional, flags: OptFlag::NONE,
    });
    assert_eq!(Opt::positional((), "").required().hide_usage(OptHide::All), Opt { id: (),
      names: OptIdentifier::Single(""), value_name: None, help_string: None,
      r#type: OptType::Positional, flags: OptFlag::REQUIRED,
    });
  }

  #[test]
  #[should_panic(expected = "Help flag cannot be made required")]
  fn test_required_help_disallowed() {
    Opt::help_flag((), &["-h", "--help"]).required();
  }

  #[test]
  #[should_panic(expected = "Only flags are allowed to be help options")]
  fn test_positional_with_help_flag_disallowed() {
    Opt::positional((), "").with_help_flag();
  }

  #[test]
  #[should_panic(expected = "Only flags are allowed to be help options")]
  fn test_value_with_help_flag_disallowed() {
    Opt::value((), &[""], "").with_help_flag();
  }

  #[test]
  fn test_flag_getters() {
    const HELP: Opt<()> = Opt::help_flag((), &[""]);
    const REQUIRED: Opt<()> = Opt::positional((), "").required();
    assert!(HELP.is_help());
    assert!(!HELP.is_required());
    assert!(REQUIRED.is_required());
    assert!(!REQUIRED.is_help());
  }

  #[test]
  fn test_first_name() {
    assert_eq!(Opt::positional((), "first").first_name(), "first");
    assert_eq!(Opt::flag((), &["first", "second"]).first_name(), "first");
  }

  #[test]
  fn test_first_long_name() {
    assert_eq!(Opt::positional((), "--long").first_long_name(), Some("--long"));
    assert_eq!(Opt::positional((), "-long").first_long_name(), Some("-long"));
    assert_eq!(Opt::positional((), "--l").first_long_name(), Some("--l"));
    assert_eq!(Opt::positional((), "-s").first_long_name(), None);
    assert_eq!(Opt::flag((), &["-s", "--long"]).first_long_name(), Some("--long"));
  }

  #[test]
  fn test_first_short_name() {
    assert_eq!(Opt::positional((), "-s").first_short_name(), Some("-s"));
    assert_eq!(Opt::positional((), "--long").first_short_name(), None);
    assert_eq!(Opt::positional((), "--").first_short_name(), None);
    assert_eq!(Opt::positional((), "-lo").first_short_name(), None);
    assert_eq!(Opt::positional((), "--l").first_short_name(), None);
    assert_eq!(Opt::flag((), &["--long", "-s"]).first_short_name(), Some("-s"));
  }

  #[test]
  fn test_first_short_name_char() {
    assert_eq!(Opt::positional((), "-s").first_short_name_char(), Some('s'));
    assert_eq!(Opt::positional((), "--long").first_short_name_char(), None);
    assert_eq!(Opt::positional((), "--").first_short_name_char(), None);
    assert_eq!(Opt::positional((), "-lo").first_short_name_char(), None);
    assert_eq!(Opt::positional((), "--l").first_short_name_char(), None);
    assert_eq!(Opt::flag((), &["--long", "-s"]).first_short_name_char(), Some('s'));
  }

  #[test]
  fn test_match_name() {
    assert_eq!(Opt::flag((), &["--one", "--two", "--threee", "--three"])
      .match_name("--three", 0), Some("--three"));
    assert_eq!(Opt::flag((), &["--one", "--two", "--threee"])
      .match_name("--three", 0), None);
    assert_eq!(Opt::flag((), &["/one", "/two", "/three", "/four"])
      .match_name("-three", 1), Some("/three"));
    assert_eq!(Opt::positional((), "-s").match_name("-s", 1), Some("-s"));

    assert_eq!(Opt::flag((), &["-x", "-s"]).match_name("-s", 2), None);
    assert_eq!(Opt::positional((), "-x").match_name("-s", 2), None);
  }
}
