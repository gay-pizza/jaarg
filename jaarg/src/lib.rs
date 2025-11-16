/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_std]

mod const_utf8;
mod ordered_bitset;

mod option;
mod options;
mod argparse;
mod help;

pub use option::*;
pub use options::*;
pub use argparse::*;
pub use help::*;

#[cfg(feature = "alloc")]
pub mod alloc;
#[cfg(feature = "std")]
pub mod std;
