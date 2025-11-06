/* basic_nostd - jaarg example program using parse in `no_std`
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#![no_std]
#![no_main]

extern crate alloc;

use jaarg::{
  ErrorUsageWriter, ErrorUsageWriterContext, HelpWriter, HelpWriterContext, Opt, Opts,
  ParseControl, ParseResult, StandardErrorUsageWriter, StandardFullHelpWriter
};
use jaarg_nostd::{print, println, harness::ExitCode, simplepathbuf::SimplePathBuf};

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn safe_main(args: &[&str]) -> ExitCode {
  // Variables for arguments to fill
  let mut file = SimplePathBuf::default();
  let mut out: Option<SimplePathBuf> = None;
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

  // Parse command-line arguments from argv
  match OPTIONS.parse(
    SimplePathBuf::from(*args.first().unwrap()).basename(),
    args.iter().skip(1),
    |program_name, id, _opt, _name, arg| {
      match id {
        Arg::Help => {
          let ctx = HelpWriterContext { options: &OPTIONS, program_name };
          print!("{}", StandardFullHelpWriter::<'_, Arg>::new(ctx));
          return Ok(ParseControl::Quit);
        }
        Arg::Number => { number = str::parse(arg)?; }
        Arg::File   => { file = arg.into(); }
        Arg::Out    => { out = Some(arg.into()); }
      }
      Ok(ParseControl::Continue)
    }, |program_name, error| {
      let ctx = ErrorUsageWriterContext { options: &OPTIONS, program_name, error };
      print!("{}", StandardErrorUsageWriter::<'_, Arg>::new(ctx));
    }
  ) {
    ParseResult::ContinueSuccess => (),
    ParseResult::ExitSuccess => { return ExitCode::SUCCESS; }
    ParseResult::ExitError => { return ExitCode::FAILURE; }
  }

  // Print the result variables
  println!("{file} -> {out} (number: {number})",
    out = out.unwrap_or(file.with_extension("out")));

  ExitCode::SUCCESS
}
