# jaarg argument parser library #

Dependency-free, const (mostly), no magic macros, `no_std` & no alloc (though nicer with those).
Some say it can parse your arguments.

### Obligatory fancy banners ###
<div>
 <a href="https://crates.io/crates/jaarg">
  <img src="https://img.shields.io/crates/v/jaarg.svg?logo=rust&style=for-the-badge" alt="Crates version" />
 </a>
 <a href="#licensing">
  <img src="https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-green.svg?style=for-the-badge" alt="MIT OR Apache-2.0 License" />
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
```

### Changelog ###

<!-- main: -->

v0.2.0:
 * Change licence from `MIT` to `MIT OR Apache-2.0`.
 * Moved `Opts::parse_map` into newly introduced `alloc` crate, making it accessible for `no_std` users.
 * More generic & flexible help API: removed forced newline, moved error writer to `StandardErrorUsageWriter`,
    generalised "Usage" line in standard full writer, enough public constructs to roll a custom help writer.
 * Added the ability to exclude options from short usage, full help, or both.
 * More tests for validating internal behaviour & enabled CI on GitHub.
 * Added new `no_std` examples.

v0.1.1:
 * Fixed incorrect error message format for coerced parsing errors.
 * Cleaned up docstring formatting.
 * Added basic example.

v0.1.0:
 * Initial release.

### Roadmap ###

Near future:
 * More control over parsing behaviour (getopt style, no special casing shorts for Windows style flags, etc.)
 * More practical examples.

Long term:
 * Strategy for handling exclusive argument groups.
 * Make use of const traits when they land to improve table setup.

### Projects using jaarg (very cool) ###
<!-- soon... * [Sprout bootloader](https://github.com/edera-dev/sprout) -->
 * [lbminfo](https://github.com/ScrelliCopter/colourcyclinginthehousetonight/tree/main/lbminfo)

### Licensing ###

jaarg is dual-licensed under either the [MIT](LICENSE.MIT) or [Apache 2.0](LICENSE.Apache-2.0) licences.
Pick whichever works best for your project.
