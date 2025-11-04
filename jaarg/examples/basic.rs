/* basic - jaarg example program using parse_easy
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

use jaarg::{Opt, Opts, ParseControl, ParseResult};
use std::path::PathBuf;

fn main() {
  // Variables for arguments to fill
  let mut file = PathBuf::new();
  let mut out: Option<PathBuf> = None;
  let mut number = 0;

  // Set up arguments table
  enum Arg { Help, Number, File, Out }
  const OPTIONS: Opts<Arg> = Opts::new(&[
    Opt::help_flag(Arg::Help, &["-h", "--help"]).help_text("Show this help and exit."),
    Opt::value(Arg::Number, &["-n", "--number"], "value")
      .help_text("Optionally specify a number (default: 0)"),
    Opt::positional(Arg::File, "file").required()
      .help_text("Input file."),
    Opt::positional(Arg::Out, "out")
      .help_text("Output destination (optional).")
  ]).with_description("My simple utility.");

  // Parse command-line arguments from `std::env::args()`
  match OPTIONS.parse_easy(|program_name, id, _opt, _name, arg| {
    match id {
      Arg::Help => {
        OPTIONS.print_full_help(program_name);
        return Ok(ParseControl::Quit);
      }
      Arg::Number => { number = str::parse(arg)?; }
      Arg::File   => { file = arg.into(); }
      Arg::Out    => { out = Some(arg.into()); }
    }
    Ok(ParseControl::Continue)
  }) {
    ParseResult::ContinueSuccess => (),
    ParseResult::ExitSuccess     => std::process::exit(0),
    ParseResult::ExitError       => std::process::exit(1),
  }

  // Print the result variables
  println!("{file:?} -> {out:?} (number: {number:?})",
    out = out.unwrap_or(file.with_extension("out")));
}
