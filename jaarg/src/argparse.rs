/* jaarg - Argument parser
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

use crate::{Opt, Opts};
use crate::option::OptType;
use crate::options::RequiredParamsBitSet;

/// Enum describing the result of parsing arguments, and how the program should behave.
#[derive(Debug)]
pub enum ParseResult {
  /// Parsing succeeded and program execution should continue.
  ContinueSuccess,
  /// Parsing succeeded and program should exit with success (eg; `exit(0)`).
  ExitSuccess,
  /// There was an error while parsing and program should exit with failure (eg; `exit(1)`).
  ExitFailure,
}

/// Execution control for parser handlers.
pub enum ParseControl {
  /// Continue parsing arguments
  Continue,
  /// Tell the parser to stop consuming tokens (treat as end of token stream)
  Stop,
  /// Tell the parser to stop parsing and quit early, this will skip end of parsing checks
  Quit,
}

#[derive(Debug)]
pub struct ParseHandlerContext<'a, 'name, ID: 'static> {
  /// Name of the program, for printing statuses to the user.
  pub program_name: &'name str,
  /// The generic argument ID that was matched.
  pub id: &'a ID,
  /// The option that was matched by the parser.
  pub option: &'a Opt<ID>,
  /// The name of the argument parameter that was matched,
  /// for option parameters this is the token supplied by the user.
  pub name: &'a str,
  /// The argument provided to positional arguments and value options (will always be Some), or None for flags.
  pub arg: Option<&'a str>,
}

/// Result type used by the handler passed to the parser.
pub(crate) type HandlerResult<'a, T> = core::result::Result<T, ParseError<'a>>;

#[derive(Debug)]
pub enum ParseError<'t> {
  UnknownOption(&'t str),
  UnexpectedToken(&'t str),
  ExpectArgument(&'t str),
  UnexpectedArgument(&'t str),
  ArgumentError(&'static str, &'t str, ParseErrorKind),
  //TODO
  //Exclusive(&'static str, &'a str),
  RequiredPositional(&'static str),
  RequiredParameter(&'static str),
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
      Self::RequiredParameter(o) => write!(f, "Missing required option '{o}'"),
    }
  }
}

/// Convenience coercion for dealing with integer parsing errors.
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

/// Convenience coercion for dealing with floating-point parsing errors.
impl From<core::num::ParseFloatError> for ParseError<'_> {
  fn from(_err: core::num::ParseFloatError) -> Self {
    // HACK: The empty option & argument fields will be fixed up by the parser
    // NOTE: Unlike ParseIntError, ParseFloatError does not expose kind publicly yet
    Self::ArgumentError("", "", ParseErrorKind::InvalidFloat)
  }
}

impl core::error::Error for ParseError<'_> {}

/// Internal state tracked by the parser.
struct ParserState<ID: 'static> {
  positional_index: usize,
  expects_arg: Option<(&'static str, &'static Opt<ID>)>,
  required_param_presences: RequiredParamsBitSet,
}

impl<ID> Default for ParserState<ID> {
  fn default() -> Self {
    Self {
      positional_index: 0,
      expects_arg: None,
      required_param_presences: Default::default(),
    }
  }
}

impl<ID: 'static> Opts<ID> {
  /// Parses an iterator of strings as argument tokens.
  pub fn parse<'a, S: AsRef<str> + 'a, I: Iterator<Item = S>>(&self, program_name: &str, args: I,
    mut handler: impl FnMut(ParseHandlerContext<ID>) -> HandlerResult<'a, ParseControl>,
    error: impl FnOnce(&str, ParseError),
  ) -> ParseResult {
    let mut state = ParserState::default();
    for arg in args {
      // Fetch the next token
      match self.next(&mut state, arg.as_ref(), program_name, &mut handler) {
        Ok(ParseControl::Continue) => {}
        Ok(ParseControl::Stop) => { break; }
        Ok(ParseControl::Quit) => { return ParseResult::ExitSuccess; }
        Err(err) => {
          // Call the error handler
          error(program_name, err);
          return ParseResult::ExitFailure;
        }
      }
    }

    self.validate_state(program_name, state, error)
  }

  /// Parses a slice of strings as argument tokens.
  /// Like [Opts::parse] but allows borrowing argument tokens outside the handler.
  pub fn parse_slice<'opts, 't, S: AsRef<str>>(&'opts self, program_name: &str, args: &'t [S],
    mut handler: impl FnMut(ParseHandlerContext<'opts, '_, ID>) -> HandlerResult<'opts, ParseControl>,
    error: impl FnOnce(&str, ParseError),
  ) -> ParseResult where 't: 'opts {
    let mut state = ParserState::default();
    for arg in args {
      // Fetch the next token
      match self.next_borrow(&mut state, arg.as_ref(), program_name, &mut handler) {
        Ok(ParseControl::Continue) => {}
        Ok(ParseControl::Stop) => { break; }
        Ok(ParseControl::Quit) => { return ParseResult::ExitSuccess; }
        Err(err) => {
          // Call the error handler
          error(program_name, err);
          return ParseResult::ExitFailure;
        }
      }
    }

    self.validate_state(program_name, state, error)
  }

  fn validate_state(&self, program_name: &str, mut state: ParserState<ID>, error: impl FnOnce(&str, ParseError)
  ) -> ParseResult {
    // Ensure that value options are provided a value
    if let Some((name, _)) = state.expects_arg.take() {
      error(program_name, ParseError::ExpectArgument(name));
      return ParseResult::ExitFailure;
    }

    // Ensure that all required arguments have been provided
    let mut required_flag_idx = 0;
    for (i, option) in self.iter().enumerate() {
      match option.r#type {
        OptType::Positional => if i >= state.positional_index && option.is_required() {
          error(program_name, ParseError::RequiredPositional(option.first_name()));
          return ParseResult::ExitFailure;
        }
        OptType::Flag | OptType::Value => if option.is_required() {
          if !state.required_param_presences.get(required_flag_idx) {
            error(program_name, ParseError::RequiredParameter(option.first_name()));
            return ParseResult::ExitFailure;
          }
          required_flag_idx += 1;
        }
      }
    }

    // All arguments parsed successfully
    ParseResult::ContinueSuccess
  }

  /// Parse the next token in the argument stream.
  fn next<'r, 't>(&self, state: &mut ParserState<ID>, token: &'t str, program_name: &str,
    handler: &mut impl FnMut(ParseHandlerContext<ID>) -> HandlerResult<'r, ParseControl>
  ) -> HandlerResult<'t, ParseControl> where 'r: 't {
    let mut call_handler = |option: &Opt<ID>, name, value| {
      match handler(ParseHandlerContext{ program_name, id: &option.id, option, name, arg: value }) {
        // HACK: Ensure the string fields are set properly, because coerced
        //       ParseIntError/ParseFloatError will have the string fields blanked.
        Err(ParseError::ArgumentError("", "", kind))
          => Err(ParseError::ArgumentError(name, value.unwrap(), kind)),
        Err(err) => Err(err),
        Ok(ctl) => Ok(ctl),
      }
    };

    // If the previous token is expecting an argument, ie: value a value option
    //  was matched and didn't have an equals sign separating a value,
    //  then call the handler here.
    if let Some((name, option)) = state.expects_arg.take() {
      call_handler(option, name, Some(token))
    } else {
      // Check if the next argument token starts with an option flag
      if self.flag_chars.chars().any(|c| token.starts_with(c)) {
        // Value options can have their value delineated by an equals sign or with whitespace.
        // In the latter case; the value will be in the next token.
        let (option_str, value_str) = token.split_once("=")
          .map_or((token, None), |(k, v)| (k, Some(v)));

        // Keep track of how many required options we've seen
        let mut required_idx = 0;

        // Match a suitable option by name (ignoring the first flag character & skipping positional arguments)
        let (name, option) = self.iter()
          .filter(|opt| matches!(opt.r#type, OptType::Flag | OptType::Value)).find_map(|opt| {
            if let Some(name) = opt.match_name(option_str, 1) {
              Some((name, opt))
            } else {
              if opt.is_required() {
                required_idx += 1
              }
              None
            }
          }).ok_or(ParseError::UnknownOption(option_str))?;

        // Mark required option as visited
        if option.is_required() {
          state.required_param_presences.insert(required_idx, true);
        }

        match (&option.r#type, value_str) {
          // Call handler for flag-only options
          (OptType::Flag, None) => call_handler(option, name, None),
          // Value was provided this token, so call the handler right now
          (OptType::Value, Some(value)) => call_handler(option, name, Some(value)),
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
          if matches!(option.r#type, OptType::Positional) {
            call_handler(option, option.first_name(), Some(token))?;
            state.positional_index += i + 1;
            return Ok(ParseControl::Continue);
          }
        }
        Err(ParseError::UnexpectedToken(token))
      }
    }
  }

  /// I absolutely hate that this needs to be DUPLICATED
  fn next_borrow<'opts, 't>(&'opts self, state: &mut ParserState<ID>, token: &'t str, program_name: &str,
    handler: &mut impl FnMut(ParseHandlerContext<'opts, '_, ID>) -> HandlerResult<'opts, ParseControl>
  ) -> HandlerResult<'opts, ParseControl> where 't: 'opts {
    let mut call_handler = |option: &'opts Opt<ID>, name, value| {
      match handler(ParseHandlerContext{ program_name, id: &option.id, option, name, arg: value }) {
        // HACK: Ensure the string fields are set properly, because coerced
        //       ParseIntError/ParseFloatError will have the string fields blanked.
        Err(ParseError::ArgumentError("", "", kind))
          => Err(ParseError::ArgumentError(name, value.unwrap(), kind)),
        Err(err) => Err(err),
        Ok(ctl) => Ok(ctl),
      }
    };

    // If the previous token is expecting an argument, ie: value a value option
    //  was matched and didn't have an equals sign separating a value,
    //  then call the handler here.
    if let Some((name, option)) = state.expects_arg.take() {
      call_handler(option, name, Some(token))
    } else {
      // Check if the next argument token starts with an option flag
      if self.flag_chars.chars().any(|c| token.starts_with(c)) {
        // Value options can have their value delineated by an equals sign or with whitespace.
        // In the latter case; the value will be in the next token.
        let (option_str, value_str) = token.split_once("=")
          .map_or((token, None), |(k, v)| (k, Some(v)));

        // Keep track of how many required options we've seen
        let mut required_idx = 0;

        // Match a suitable option by name (ignoring the first flag character & skipping positional arguments)
        let (name, option) = self.iter()
          .filter(|opt| matches!(opt.r#type, OptType::Flag | OptType::Value)).find_map(|opt| {
            if let Some(name) = opt.match_name(option_str, 1) {
              Some((name, opt))
            } else {
              if opt.is_required() {
                required_idx += 1
              }
              None
            }
          }).ok_or(ParseError::UnknownOption(option_str))?;

        // Mark required option as visited
        if option.is_required() {
          state.required_param_presences.insert(required_idx, true);
        }

        match (&option.r#type, value_str) {
          // Call handler for flag-only options
          (OptType::Flag, None) => call_handler(option, name, None),
          // Value was provided this token, so call the handler right now
          (OptType::Value, Some(value)) => call_handler(option, name, Some(value)),
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
          if matches!(option.r#type, OptType::Positional) {
            call_handler(option, option.first_name(), Some(token))?;
            state.positional_index += i + 1;
            return Ok(ParseControl::Continue);
          }
        }
        Err(ParseError::UnexpectedToken(token))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  enum ArgID { One, Two, Three, Four, Five }
  const OPTIONS: Opts<ArgID> = Opts::new(&[
    Opt::positional(ArgID::One, "one"),
    Opt::flag(ArgID::Two, &["--two"]),
    Opt::value(ArgID::Three, &["--three"], "value"),
    Opt::value(ArgID::Four, &["--four"], "value"),
    Opt::value(ArgID::Five, &["--five"], "value"),
  ]);
  const ARGUMENTS: &[&str] = &["one", "--two", "--three=three", "--five=", "--four", "four"];

  #[test]
  fn test_parse() {
    extern crate alloc;
    use alloc::string::String;

    let mut one: Option<String> = None;
    let mut two = false;
    let mut three: Option<String> = None;
    let mut four: Option<String> = None;
    let mut five: Option<String> = None;
    assert!(matches!(OPTIONS.parse("", ARGUMENTS.iter(), |ctx| {
      match ctx.id {
        ArgID::One =>   { one = Some(ctx.arg.unwrap().into()); }
        ArgID::Two =>   { two = true; }
        ArgID::Three => { three = Some(ctx.arg.unwrap().into()); }
        ArgID::Four =>  { four = Some(ctx.arg.unwrap().into()); }
        ArgID::Five =>  { five = Some(ctx.arg.unwrap().into()); }
      }
      Ok(ParseControl::Continue)
    }, |_, error| {
      panic!("unreachable: {error:?}");
    }), ParseResult::ContinueSuccess));

    assert_eq!(one, Some("one".into()));
    assert!(two);
    assert_eq!(three, Some("three".into()));
    assert_eq!(four, Some("four".into()));
    assert_eq!(five, Some("".into()));
  }

  #[test]
  fn test_parse_slice() {
    let mut one: Option<&str> = None;
    let mut two = false;
    let mut three: Option<&str> = None;
    let mut four: Option<&str> = None;
    let mut five: Option<&str> = None;
    assert!(matches!(OPTIONS.parse_slice("", &ARGUMENTS, |ctx| {
      match ctx.id {
        ArgID::One =>   { one = ctx.arg; }
        ArgID::Two =>   { two = true; }
        ArgID::Three => { three = ctx.arg; }
        ArgID::Four =>  { four = ctx.arg; }
        ArgID::Five =>  { five = ctx.arg; }
      }
      Ok(ParseControl::Continue)
    }, |_, error| {
      panic!("unreachable: {error:?}");
    }), ParseResult::ContinueSuccess));

    assert_eq!(one, Some("one"));
    assert!(two);
    assert_eq!(three, Some("three"));
    assert_eq!(four, Some("four"));
    assert_eq!(five, Some(""));
  }
}
