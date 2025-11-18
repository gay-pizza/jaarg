/* basic_nostd - jaarg example program using parse in `no_std`
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

#![no_std]
#![no_main]

use jaarg::{
  ErrorUsageWriter, ErrorUsageWriterContext, HelpWriter, HelpWriterContext, Opt, Opts,
  ParseControl, ParseResult, StandardErrorUsageWriter, StandardFullHelpWriter
};
use jaarg_nostd::{print, println, harness::ExitCode, simplepathbuf::SimplePathBuf};

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn safe_main(args: &[&str]) -> ExitCode {
  // Variables for arguments to fill
  let mut file: Option<&str> = None;
  let mut out: Option<&str> = None;
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
  match OPTIONS.parse_slice(
    SimplePathBuf::from(*args.first().unwrap()).basename(),
    &args[1..], |ctx| {
      match ctx.id {
        Arg::Help => {
          let ctx = HelpWriterContext { options: &OPTIONS, program_name: ctx.program_name };
          print!("{}", StandardFullHelpWriter::<'_, Arg>::new(ctx));
          return Ok(ParseControl::Quit);
        }
        Arg::Number => { number = str::parse(ctx.arg.unwrap())?; }
        Arg::File   => { file = ctx.arg; }
        Arg::Out    => { out = ctx.arg; }
      }
      Ok(ParseControl::Continue)
    }, |program_name, error| {
      let ctx = ErrorUsageWriterContext { options: &OPTIONS, program_name, error };
      print!("{}", StandardErrorUsageWriter::<'_, Arg>::new(ctx));
    }
  ) {
    ParseResult::ContinueSuccess => (),
    ParseResult::ExitSuccess => { return ExitCode::SUCCESS; }
    ParseResult::ExitFailure => { return ExitCode::FAILURE; }
  }

  // Print the result variables
  let file = SimplePathBuf::from(file.unwrap());
  println!("{file} -> {out} (number: {number})",
    out = out.map_or(file.with_extension("out"), |out| SimplePathBuf::from(out)));

  ExitCode::SUCCESS
}
