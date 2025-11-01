/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

/// Fully const fn nostd UTF-8 character iterator.
/// Assumes a well-formed UTF-8 input string. Doesn't take into account graphemes.
pub(crate) struct CharIterator<'a> {
  bytes: &'a [u8],
  index: usize
}

impl<'a> CharIterator<'a> {
  /// Create a char iterator from an immutable string slice.
  #[inline]
  pub(crate) const fn from(value: &'a str) -> Self {
    Self {
      bytes: value.as_bytes(),
      index: 0,
    }
  }

}

impl CharIterator<'_> {
  /// Gets a count of the number of Unicode characters (not graphemes) in the string.
  pub(crate) const fn count(self) -> usize {
    let len = self.bytes.len();
    let mut count = 0;
    let mut i = 0;
    while i < len {
      // Count all bytes that don't start with 0b10xx_xxxx (UTF-8 continuation byte)
      if (self.bytes[i] as i8) >= -64 {
        count += 1;
      }
      i += 1;
    }
    count
  }

  /// Gets the next character in a well-formed UTF-8 string, or None for end of string or errors.
  pub(crate) const fn next(&mut self) -> Option<char> {
    /// UTF-8 2-byte flag bits
    const MULTIBYTE_2: u8 = 0b1100_0000;
    /// UTF-8 3-byte flag bits
    const MULTIBYTE_3: u8 = 0b1110_0000;
    /// UTF-8 4-byte flag bits
    const MULTIBYTE_4: u8 = 0b1111_0000;

    /// Mask for UTF-8 2-byte flag bits
    const MULTIBYTE_2_MASK: u8 = 0b1110_0000;
    /// Mask for UTF-8 3-byte flag bits
    const MULTIBYTE_3_MASK: u8 = 0b1111_0000;
    /// Mask for UTF-8 4-byte flag bits
    const MULTIBYTE_4_MASK: u8 = 0b1111_1000;

    /// UTF-8 continuation flag bits
    const CONTINUATION: u8 = 0b1000_0000;
    /// Mask for the UTF-8 continuation flag bits
    const CONTINUATION_MASK: u8 = 0b1100_0000;

    /// Checks if a byte begins with the UTF-8 continuation bits
    #[inline] const fn is_continuation(b: u8) -> bool { b & CONTINUATION_MASK == CONTINUATION }
    /// Gets the value bits of a UTF-8 continuation byte as u32
    #[inline] const fn cont_bits(b: u8) -> u32 { (b & !CONTINUATION_MASK) as u32 }

    // Return early if we reached the end of the string
    if self.index >= self.bytes.len() {
      return None;
    }

    let byte0 = self.bytes[self.index];

    // Get the length of the next multibyte UTF-8 character
    let len = match byte0 {
      ..0x80 => 1,
      _ if (byte0 & MULTIBYTE_2_MASK) == MULTIBYTE_2 => 2,
      _ if (byte0 & MULTIBYTE_3_MASK) == MULTIBYTE_3 => 3,
      _ if (byte0 & MULTIBYTE_4_MASK) == MULTIBYTE_4 => 4,
      _ => {
        return None;
      }
    };

    // Return early for incomplete sequences
    if len > self.bytes.len() - self.index {
      return None;
    }

    // Try to read the next multibyte character
    let Some(result) = (match len {
      1 => Some(byte0 as char),
      2 if is_continuation(self.bytes[self.index + 1])
      => {
        let cp = (((byte0 & !MULTIBYTE_2_MASK) as u32) << 6) | cont_bits(self.bytes[self.index + 1]);
        char::from_u32(cp)
      },
      3 if is_continuation(self.bytes[self.index + 1])
        && is_continuation(self.bytes[self.index + 2])
      => {
        let cp = (((byte0 & !MULTIBYTE_3_MASK) as u32) << 12)
          | (cont_bits(self.bytes[self.index + 1]) << 6)
          | cont_bits(self.bytes[self.index + 2]);
        char::from_u32(cp)
      }
      4 if is_continuation(self.bytes[self.index + 1])
        && is_continuation(self.bytes[self.index + 2])
        && is_continuation(self.bytes[self.index + 3])
      => {
        let cp = (((byte0 & !MULTIBYTE_4_MASK) as u32) << 18)
          | (cont_bits(self.bytes[self.index + 1]) << 12)
          | (cont_bits(self.bytes[self.index + 2]) << 6)
          | cont_bits(self.bytes[self.index + 3]);
        char::from_u32(cp)
      }
      _ => None,
    }) else {
      return None
    };

    // Advance the internal character index and return success
    self.index += len;
    Some(result)
  }
}
