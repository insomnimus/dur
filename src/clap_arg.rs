use std::ffi::OsStr;

use clap::{
	builder::{
		StringValueParser,
		TypedValueParser,
		ValueParserFactory,
	},
	error::{
		ContextKind,
		ContextValue,
		ErrorKind,
	},
	Arg,
	Command,
};

use crate::Duration;

#[derive(Clone, Debug)]
pub struct DurationParser;

impl ValueParserFactory for Duration {
	type Parser = DurationParser;

	fn value_parser() -> Self::Parser {
		DurationParser
	}
}

impl TypedValueParser for DurationParser {
	type Value = Duration;

	fn parse_ref(
		&self,
		cmd: &Command,
		arg: Option<&Arg>,
		value: &OsStr,
	) -> Result<Self::Value, clap::Error> {
		let s = StringValueParser::new().parse_ref(cmd, arg, value)?;
		crate::parse(&s).map_err(move |e| {
			let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
			if let Some(arg) = arg {
				err.insert(
					ContextKind::InvalidArg,
					ContextValue::String(arg.to_string()),
				);
			}
			err.insert(ContextKind::InvalidValue, ContextValue::String(s));
			err.insert(ContextKind::Custom, ContextValue::String(e.to_string()));
			err
		})
	}
}
