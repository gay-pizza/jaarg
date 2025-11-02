# jaarg argument parser library #
nostd & (mostly) const, some say it can parse your arguments.

### Obligatory fancy banners ###
<div>
 <a href="https://crates.io/crates/jaarg">
  <img src="https://img.shields.io/crates/v/jaarg.svg?logo=rust&style=for-the-badge" alt="Crates version" />
 </a>
 <a href="LICENSE">
  <img src="https://img.shields.io/badge/license-MIT-green.svg?style=for-the-badge" alt="MIT License" />
 </a>
</div>

### Example usage ###
```rust
// Variables for arguments to fill
let mut file = PathBuf::new();
let mut out: Option<PathBuf> = None;
let mut number = 0;

// Set up arguments table
enum Arg { Help, Number, File, Out }
const OPTIONS: Opts<Arg> = Opts::new(&[
  Opt::help_flag(Arg::Help, &["-h", "--help"]).help_text("Show this help and exit."),
  Opt::value(Arg::Number, &["-n", "--number"], "value").help_text("Optionally specify a number (default: 0)"),
  Opt::positional(Arg::File, "file").required().help_text("Input file."),
  Opt::positional(Arg::Out, "out").help_text("Output destination (optional).")
]).with_description("My simple utility.");

// Parse the arguments
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
```

### Changelog ###

<!-- main: -->

v0.1.0:
 * Initial release.

### Projects using jaarg (very cool) ###
<!-- soon... * [Sprout bootloader](https://github.com/edera-dev/sprout) -->
 * [lbminfo](https://github.com/ScrelliCopter/colourcyclinginthehousetonight/tree/main/lbminfo)
