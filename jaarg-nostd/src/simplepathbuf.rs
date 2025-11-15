/* jaarg-nostd - Minimal harness to run examples in no_std on desktop
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use alloc::format;
use alloc::string::String;
use core::fmt::{Display, Formatter};

/// Dirty and simple path buffer that's good enough for the `no_std` examples, not for production use.
#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SimplePathBuf(String);

impl<S: AsRef<str>> From<S> for SimplePathBuf where String: From<S> {
  fn from(value: S) -> Self {
    Self(value.into())
  }
}

impl Display for SimplePathBuf {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl SimplePathBuf {
  #[inline(always)]
  fn path_predicate(c: char) -> bool {
    #[cfg(target_family = "windows")]
    if c == '\\' { return true; }
    c == '/'
  }

  pub fn with_extension(&self, ext: &str) -> Self {
    let dir_sep = self.0.rfind(Self::path_predicate)
      .map_or(0, |n| n + 1);
    let without_ext: &str = self.0[dir_sep..].rfind('.')
      .map_or(&self.0, |ext_sep_rel| {
        if ext_sep_rel == 0 { return &self.0; }
        let ext_sep = dir_sep + ext_sep_rel;
        &self.0[..ext_sep]
      });
    Self(format!("{without_ext}.{ext}"))
  }

  pub fn basename(&self) -> &str {
    self.0.trim_end_matches(|c| Self::path_predicate(c) || c == '.')
      .rsplit_once(Self::path_predicate)
      .map_or(&self.0, |(_, base)| base)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_with_extension() {
    assert_eq!(
      SimplePathBuf::from("name.ext").with_extension("new"),
      SimplePathBuf::from("name.new"));
    assert_eq!(
      SimplePathBuf::from("/path/name.ext").with_extension("new"),
      SimplePathBuf::from("/path/name.new"));
    assert_eq!(
      SimplePathBuf::from("/path.ext/name").with_extension("new"),
      SimplePathBuf::from("/path.ext/name.new"));
    assert_eq!(
      SimplePathBuf::from("/path.ext/.name").with_extension("new"),
      SimplePathBuf::from("/path.ext/.name.new"));
  }

  #[test]
  fn test_basename() {
    assert_eq!(SimplePathBuf::from("name.ext").basename(), "name.ext");
    assert_eq!(SimplePathBuf::from("/path/name.ext").basename(), "name.ext");
    assert_eq!(SimplePathBuf::from("/path/name/").basename(), "name");
    assert_eq!(SimplePathBuf::from("/path/name/.").basename(), "name");
    assert_eq!(SimplePathBuf::from("/path/name/.//").basename(), "name");
  }
}
