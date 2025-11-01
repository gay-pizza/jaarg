/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#[derive(Debug, Copy, Clone)]
enum OptType {
  Positional,
  Flag,
  Value,
}

#[derive(Debug)]
enum OptIdentifier {
  Single(&'static str),
  Multi(&'static[&'static str]),
}

/// Represents an option argument or positional argument to be parsed
#[derive(Debug)]
pub struct Opt<ID> {
  id: ID,
  names: OptIdentifier,
  value_name: Option<&'static str>,
  help_string: &'static str,
  r#type: OptType,
  required: bool,
}

// TODO: Improve this interface by making the name field take AsOptIdentifier when const traits are stabilised
impl<ID> Opt<ID> {
  /// A positional argument that is parsed sequentially without being invoked by an option flag
  pub const fn positional(id: ID, name: &'static str, help_string: &'static str) -> Self {
    Self { id, names: OptIdentifier::Single(name), value_name: None, help_string, r#type: OptType::Positional, required: false }
  }
  /// A required positional argument that is parsed sequentially without being invoked by an option flag
  pub const fn positional_required(id: ID, name: &'static str, help_string: &'static str) -> Self {
    Self { id, names: OptIdentifier::Single(name), value_name: None, help_string, r#type: OptType::Positional, required: true }
  }
  /// A flag-type option that takes no value
  pub const fn flag(id: ID, names: &'static[&'static str], help_string: &'static str) -> Self {
    Self { id, names: OptIdentifier::Multi(names), value_name: None, help_string, r#type: OptType::Flag, required: false }
  }
  /// A required flag-type option that takes no value
  pub const fn flag_required(id: ID, names: &'static[&'static str], help_string: &'static str) -> Self {
    Self { id, names: OptIdentifier::Multi(names), value_name: None, help_string, r#type: OptType::Flag, required: true }
  }
  /// An option argument that takes a value
  pub const fn value(id: ID, names: &'static[&'static str], value_name: &'static str, help_string: &'static str) -> Self {
    Self { id, names: OptIdentifier::Multi(names), value_name: Some(value_name), help_string, r#type: OptType::Value, required: false }
  }
  /// A required option argument that takes a value
  pub const fn value_required(id: ID, names: &'static[&'static str], value_name: &'static str, help_string: &'static str) -> Self {
    Self { id, names: OptIdentifier::Multi(names), value_name: Some(value_name), help_string, r#type: OptType::Value, required: true }
  }
}

impl<ID: 'static> Opt<ID> {
  /// Get the first name of the option
  const fn first_name(&self) -> &str {
    match self.names {
      OptIdentifier::Single(name) => name,
      OptIdentifier::Multi(names) => names.first().unwrap(),
    }
  }

  /// Get the first long option name, if one exists
  const fn first_long_name(&self) -> Option<&'static str> {
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

  /// Get the first short option name, if one exists
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

  /// Get the first applicable short option's flag character, if one exists
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
      // Can be replaced with `find_map` once iterators are const fn
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

  /// Search for a matching name in the option, offset allows to skip the first characters in the comparison
  fn match_name(&self, string: &str, offset: usize) -> Option<&'static str> {
    match self.names {
      OptIdentifier::Single(name) =>
        if name[offset..] == string[offset..] { Some(name) } else { None },
      OptIdentifier::Multi(names) =>
        names.iter().find(|name| name[offset..] == string[offset..]).map(|v| &**v),
    }
  }
}
