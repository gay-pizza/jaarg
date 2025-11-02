/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#![allow(private_bounds)]

use core::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl};

pub(crate) struct OrderedBitSet<T: OrderedBitSetStorage, const S: usize>([T; S]);

impl<T: OrderedBitSetStorage, const S: usize> Default for OrderedBitSet<T, S> {
  fn default() -> Self { Self::new() }
}

// TODO: Obvious target for improvement when const traits land
impl<T: OrderedBitSetStorage, const S: usize> OrderedBitSet<T, S> {
  /// Number of slots in the bit set.
  pub(crate) const CAPACITY: usize = T::BITS as usize * S;

  /// Creates a new, empty bit set.
  pub(crate) const fn new() -> Self { Self([T::ZERO; S]) }

  /// Sets the slot at `index` to a binary value.
  pub(crate) fn insert(&mut self, index: usize, value: bool) {
    let (array_idx, bit_idx) = self.internal_index(index);
    let bit_mask = T::from_usize(0b1) << T::from_usize(bit_idx);
    if value {
      self.0[array_idx] |= bit_mask;
    } else {
      self.0[array_idx] &= !bit_mask;
    }
  }

  /// Gets the binary value at slot `index`.
  pub(crate) fn get(&self, index: usize) -> bool {
    let (array_idx, bit_idx) = self.internal_index(index);
    let bit_mask = T::from_usize(0b1) << T::from_usize(bit_idx);
    (self.0[array_idx] & bit_mask) != T::from_usize(0)
  }

  #[inline]
  const fn internal_index(&self, index: usize) -> (usize, usize) {
    debug_assert!(index < Self::CAPACITY, "Index out of range");
    let array_idx = index >> T::SHIFT;
    let bit_idx = index & T::MASK;
    (array_idx, bit_idx)
  }
}

trait OrderedBitSetStorage: core::fmt::Debug
    + Default + Copy + Clone + Eq + PartialEq
    + BitAnd<Output = Self> + Shl<Output = Self> + Not<Output = Self>
    + BitAndAssign + BitOrAssign {
  const ZERO: Self;
  const SHIFT: u32;
  const MASK: usize;
  const BITS: u32;
  fn from_usize(value: usize) -> Self;
}

macro_rules! impl_bitset_storage {
  ($t:ty, $b:expr) => {
    impl OrderedBitSetStorage for $t {
      const ZERO: $t = 0;
      const SHIFT: u32 = $b.ilog2();
      const MASK: usize = $b as usize - 1;
      const BITS: u32 = $b;
      #[inline(always)]
      fn from_usize(value: usize) -> $t { value as $t }
    }
  };
}

impl_bitset_storage!(u8,  u8::BITS);
impl_bitset_storage!(u16, u16::BITS);
impl_bitset_storage!(u32, u32::BITS);
impl_bitset_storage!(u64, u64::BITS);
impl_bitset_storage!(u128, u128::BITS);


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bitset_storage() {
    fn harness<T: OrderedBitSetStorage + core::fmt::Debug>(bits_expect: u32, shift_expect: u32) {
      assert_eq!(T::ZERO, T::from_usize(0));
      assert_eq!(T::SHIFT, shift_expect);
      assert_eq!(T::MASK, bits_expect as usize - 1);
      assert_eq!(T::BITS, bits_expect);
    }

    harness::<u8>(8, 3);
    harness::<u16>(16, 4);
    harness::<u32>(32, 5);
    harness::<u64>(64, 6);
    harness::<u128>(128, 7);
  }

  #[test]
  fn test_ordered_bitset() {
    fn harness<T: OrderedBitSetStorage, const S: usize>(indices: &[usize]) {
      let mut bitset = OrderedBitSet::<T, S>::new();
      for &index in indices {
        bitset.insert(index, true);
      }
      for slot in 0..OrderedBitSet::<u32, 4>::CAPACITY {
        assert_eq!(bitset.get(slot), indices.contains(&slot));
      }
      for &index in indices {
        bitset.insert(index, false);
        assert!(!bitset.get(index));
      }
    }

    let indices = [1, 32, 33, 127, 44, 47, 49];
    harness::<u8, 16>(&indices);
    harness::<u16, 8>(&indices);
    harness::<u32, 4>(&indices);
    harness::<u64, 2>(&indices);
    harness::<u128, 1>(&indices);
  }
}
