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
  pub(crate) const fn new() -> Self { Self([T::ZERO; S]) }

  pub(crate) fn insert(&mut self, index: usize, value: bool) {
    let array_idx = index >> T::SHIFT;
    debug_assert!(array_idx < S, "Index out of range");
    let bit_idx = index & T::MASK;
    let bit_mask = T::from_usize(0b1) << T::from_usize(bit_idx);
    if value {
      self.0[array_idx] |= bit_mask;
    } else {
      self.0[array_idx] &= !bit_mask;
    }
  }

  pub(crate) fn get(&self, index: usize) -> bool {
    let array_idx = index >> T::SHIFT;
    debug_assert!(array_idx < S, "Index out of range");
    let bit_idx = index & T::MASK;
    let bit_mask = T::from_usize(0b1) << T::from_usize(bit_idx);
    (self.0[array_idx] & bit_mask) != T::from_usize(0)
  }
}

trait OrderedBitSetStorage: Default + Copy + Clone + Eq + PartialEq
    + BitAnd<Output = Self> + Shl<Output = Self> + Not<Output = Self>
    + BitAndAssign + BitOrAssign {
  const ZERO: Self;
  const SHIFT: u32;
  const MASK: usize;
  fn from_usize(value: usize) -> Self;
}

macro_rules! impl_bitset_storage {
  ($t:ty, $b:expr) => {
    impl OrderedBitSetStorage for $t {
      const ZERO: $t = 0;
      const SHIFT: u32 = $b.ilog2();
      const MASK: usize = $b as usize - 1;
      fn from_usize(value: usize) -> $t { value as $t }
    }
  };
}

impl_bitset_storage!(u8,  u8::BITS);
impl_bitset_storage!(u16, u16::BITS);
impl_bitset_storage!(u32, u32::BITS);
impl_bitset_storage!(u64, u64::BITS);
impl_bitset_storage!(u128, u128::BITS);
