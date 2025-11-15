/* btreemap_nostd - jaarg example program using BTreeMap in `no_std`
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

#![no_std]
#![no_main]

extern crate alloc;

use jaarg::{
  alloc::ParseMapResult, ErrorUsageWriter, ErrorUsageWriterContext, HelpWriter, HelpWriterContext,
  Opt, Opts, StandardErrorUsageWriter, StandardFullHelpWriter
};
use jaarg_nostd::{eprint, print, println, harness::ExitCode, simplepathbuf::SimplePathBuf};

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn safe_main(args: &[&str]) -> ExitCode {
  const OPTIONS: Opts<&'static str> = Opts::new(&[
    Opt::help_flag("help", &["--help"]).help_text("Show this help"),
    Opt::positional("positional", "positional").help_text("Positional argument"),
    Opt::value("value", &["-v", "--value"], "string").help_text("Value option"),
    Opt::flag("flag", &["-f", "--flag"]).help_text("Flag option"),
  ]);

  let map = match OPTIONS.parse_map(
    SimplePathBuf::from(*args.first().unwrap()).basename(),
    args.iter().skip(1),
    |program_name| {
      let ctx = HelpWriterContext { options: &OPTIONS, program_name };
      print!("{}", StandardFullHelpWriter::new(ctx));
    },
    |program_name, error| {
      let ctx = ErrorUsageWriterContext { options: &OPTIONS, program_name, error };
      eprint!("{}", StandardErrorUsageWriter::new(ctx));
    }
  ) {
    ParseMapResult::Map(map) => map,
    ParseMapResult::ExitSuccess => { return ExitCode::SUCCESS; }
    ParseMapResult::ExitFailure => { return ExitCode::FAILURE; }
  };

  println!("{:?}", map);
  ExitCode::SUCCESS
}
