/* btreemap - jaarg example program using BTreeMap
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

use jaarg::{std::ParseMapResult, Opt, Opts};
use std::process::ExitCode;

fn main() -> ExitCode {
  const OPTIONS: Opts<&'static str> = Opts::new(&[
    Opt::help_flag("help", &["--help"]).help_text("Show this help"),
    Opt::positional("positional", "positional").help_text("Positional argument"),
    Opt::value("value", &["-v", "--value"], "string").help_text("Value option"),
    Opt::flag("flag", &["-f", "--flag"]).help_text("Flag option"),
  ]);

  let map = match OPTIONS.parse_map_easy() {
    ParseMapResult::Map(map) => map,
    ParseMapResult::Exit(code) => { return code; }
  };

  println!("{:?}", map);
  ExitCode::SUCCESS
}
