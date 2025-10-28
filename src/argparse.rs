/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT
 */

/// Enum describing the result of parsing arguments, and how the program should behave.
#[derive(Debug)]
pub enum ParseResult {
  /// Parsing succeeded and program execution should continue
  ContinueSuccess,
  /// Parsing succeeded and program should exit with success (eg; std::process::ExitCode::SUCCESS)
  ExitSuccess,
  /// There was an error while parsing and program should exit with failure (eg; std::process::ExitCode::FAILURE)
  ExitError,
}

/// Execution control for the parser handler
pub enum ParseControl {
  /// Continue parsing arguments
  Continue,
  /// Tell the parser to stop consuming tokens (treat as end of token stream)
  Stop,
  /// Tell the parser to stop parsing and quit early, this will skip end of parsing checks
  Quit,
}

/// Result type used by the handler passed to the parser
type HandlerResult<'a, T> = Result<T, ParseError<'a>>;

#[derive(Debug)]
pub enum ParseError<'a> {
  UnknownOption(&'a str),
  UnexpectedToken(&'a str),
  ExpectArgument(&'a str),
  UnexpectedArgument(&'a str),
  ArgumentError(&'static str, &'a str, ParseErrorKind),
  //TODO
  //Exclusive(&'static str, &'a str),
  RequiredPositional(&'static str),
}

/// The type of parsing error
#[derive(Debug)]
pub enum ParseErrorKind {
  IntegerEmpty,
  IntegerRange,
  InvalidInteger,
  InvalidFloat,
}

impl core::fmt::Display for ParseError<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::UnknownOption(o) => write!(f, "Unrecognised option '{o}'"),
      Self::UnexpectedToken(t) => write!(f, "Unexpected positional argument '{t}'"),
      Self::ExpectArgument(o) => write!(f, "Option '{o}' requires an argument"),
      Self::UnexpectedArgument(o) => write!(f, "Flag '{o}' doesn't take an argument"),
      Self::ArgumentError(o, a, ParseErrorKind::IntegerRange)
        => write!(f, "Argument '{a}' out of range for option '{o}'"),
      Self::ArgumentError(o, a, ParseErrorKind::InvalidInteger | ParseErrorKind::InvalidFloat)
        => write!(f, "Invalid argument '{a}' for option '{o}'"),
      Self::ArgumentError(o, _, ParseErrorKind::IntegerEmpty)
        => write!(f, "Argument for option '{o}' cannot be empty"),
      //Self::Exclusive(l, r) => write!(f, "Argument {l}: not allowed with argument {r}"),
      Self::RequiredPositional(o) => write!(f, "Missing required positional argument '{o}'"),
    }
  }
}

/// Convenience coercion for dealing with integer parsing errors
impl From<core::num::ParseIntError> for ParseError<'_> {
  fn from(err: core::num::ParseIntError) -> Self {
    use core::num::IntErrorKind;
    // HACK: The empty option & argument fields will be fixed up by the parser
    Self::ArgumentError("", "", match err.kind() {
      IntErrorKind::Empty => ParseErrorKind::IntegerEmpty,
      IntErrorKind::PosOverflow | IntErrorKind::NegOverflow | IntErrorKind::Zero
        => ParseErrorKind::IntegerRange,
      IntErrorKind::InvalidDigit | _ => ParseErrorKind::InvalidInteger,
    })
  }
}

/// Convenience coercion for dealing with floating-point parsing errors
impl From<core::num::ParseFloatError> for ParseError<'_> {
  fn from(_err: core::num::ParseFloatError) -> Self {
    // HACK: The empty option & argument fields will be fixed up by the parser
    // NOTE: Unlike ParseIntError, ParseFloatError does not expose kind publicly yet
    Self::ArgumentError("", "", ParseErrorKind::InvalidFloat)
  }
}

impl core::error::Error for ParseError<'_> {}

/// Internal state tracked by the parser
struct ParserState<ID: 'static> {
  positional_index: usize,
  expects_arg: Option<(&'static str, &'static Opt<ID>)>,
}

impl<ID> Default for ParserState<ID> {
  fn default() -> Self {
    Self {
      positional_index: 0,
      expects_arg: None
    }
  }
}

impl<ID: 'static> Opts<ID> {
  /// Parse an iterator of strings as arguments
  pub fn parse<'a, S: AsRef<str> + 'a, I: Iterator<Item = S>>(&self, program_name: &str, args: I,
    mut handler: impl FnMut(&ID, &Opt<ID>, &str, &str) -> HandlerResult<'a, ParseControl>,
    error: impl FnOnce(ParseError),
  ) -> ParseResult {
    let mut state = ParserState::default();
    for arg in args {
      // Fetch the next token
      match self.next(&mut state, arg.as_ref(), &mut handler) {
        Ok(ParseControl::Continue) => {}
        Ok(ParseControl::Stop) => { break; }
        Ok(ParseControl::Quit) => { return ParseResult::ExitSuccess; }
        Err(err) => {
          // Call the error handler
          error(err);
          return ParseResult::ExitError;
        }
      }
    }

    // Ensure that value options are provided a value
    if let Some((name, _)) = state.expects_arg.take() {
      error(ParseError::ExpectArgument(name));
      return ParseResult::ExitError;
    }

    //TODO: Ensure all required parameter arguments have been provided

    // Ensure that all required positional arguments have been provided
    for option in self.options[state.positional_index..].iter() {
      if option.r#type == OptType::Positional && option.required {
        error(ParseError::RequiredPositional(option.first_name()));
        return ParseResult::ExitError;
      }
    }

    // All arguments parsed successfully
    ParseResult::ContinueSuccess
  }

  /// Parse the next token in the argument stream
  fn next<'a, 'b>(&self, state: &mut ParserState<ID>, token: &'b str,
    handler: &mut impl FnMut(&ID, &Opt<ID>, &str, &str) -> HandlerResult<'a, ParseControl>
  ) -> HandlerResult<'b, ParseControl> where 'a: 'b {
    let mut call_handler = |option: &Opt<ID>, name, value| {
      match handler(&option.id, option, name, value) {
        // HACK: ensure the string fields are set properly, because coerced
        //       ParseIntError/ParseFloatError will have the string fields blanked.
        Err(ParseError::ArgumentError("", "", kind))
          => Err(ParseError::ArgumentError(name, token, kind)),
        Err(err) => Err(err),
        Ok(ctl) => Ok(ctl),
      }
    };

    // If the previous token is expecting an argument, ie: value a value option
    //  was matched and didn't have an equals sign separating a value,
    //  then call the handler here.
    if let Some((name, option)) = state.expects_arg.take() {
      call_handler(option, name, token)
    } else {
      // Check if the next argument token starts with an option flag
      if self.flag_chars.chars().any(|c| token.starts_with(c)) {
        // Value options can have their value delineated by an equals sign or with whitespace.
        // In the latter case; the value will be in the next token.
        let (option_str, value_str) = token.split_once("=")
          .map_or((token, None), |(k, v)| (k, Some(v)));

        // Match a suitable option by name (ignoring the first flag character & skipping positional arguments)
        let (name, option) = self.options.iter()
          .filter(|opt| opt.r#type != OptType::Positional)
          .find_map(|opt| opt.match_name(option_str, 1).map(|name| (name, opt)))
          .ok_or(ParseError::UnknownOption(option_str))?;

        match (&option.r#type, value_str) {
          // Call handler for flag-only options
          (OptType::Flag, None) => call_handler(option, name, ""),
          // Value was provided this token, so call the handler right now
          (OptType::Value, Some(value)) => call_handler(option, name, value),
          // No value available in this token, delay handling to next token
          (OptType::Value, None) => {
            state.expects_arg = Some((name, option));
            Ok(ParseControl::Continue)
          }
          // Flag-only options do not support arguments
          (OptType::Flag, Some(_)) => Err(ParseError::UnexpectedArgument(option_str)),
          // Positional arguments are filtered out so this is impossible
          (OptType::Positional, _) => unreachable!("Won't parse a positional argument as an option"),
        }
      } else {
        // Find the next positional argument
        for (i, option) in self.options[state.positional_index..].iter().enumerate() {
          if option.r#type == OptType::Positional {
            handler(&option.id, option, option.first_name(), token)?;
            state.positional_index += i + 1;
            return Ok(ParseControl::Continue);
          }
        }
        Err(ParseError::UnexpectedToken(token))
      }
    }
  }
}
