/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#![no_std]

mod const_utf8;
mod ordered_bitset;

include!("option.rs");
include!("options.rs");
include!("argparse.rs");
include!("help.rs");

#[cfg(feature = "std")]
pub mod std;
