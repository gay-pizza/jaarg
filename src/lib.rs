/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#![no_std]

include!("option.rs");
include!("options.rs");
include!("argparse.rs");

#[cfg(feature = "std")]
pub mod std;
