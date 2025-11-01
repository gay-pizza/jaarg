/* btreemap - jaarg example program using BTreeMap
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

use jaarg::{std::ParseMapResult, Opt, Opts};
use std::process::ExitCode;

fn main() -> ExitCode {
  const OPTIONS: Opts<&'static str> = Opts::new(&[
    Opt::help_flag("help", &["--help"], "Show this help"),
    Opt::positional("positional", "positional", "Positional argument"),
    Opt::value("value", &["-v", "--value"], "string", "Value option"),
    Opt::flag("flag", &["-f", "--flag"], "Flag option"),
  ]);

  let map = match OPTIONS.parse_map_easy() {
    ParseMapResult::Map(map) => map,
    ParseMapResult::Exit(code) => { return code; }
  };

  println!("{:?}", map);
  ExitCode::SUCCESS
}
