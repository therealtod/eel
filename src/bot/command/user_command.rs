use crate::bot::command::user_command::UserCommand::{
    JoinMe, JoinTable, Leave, SpectateTable, StartGame,
};
use crate::hanablive::TableId;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    EmptyCommand,
    UnknownCommand(String),
    MissingArgument {
        command: String,
        arg_index: usize,
    },
    InvalidTableId {
        command: String,
        raw: String,
        source: <TableId as std::str::FromStr>::Err,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCommand => write!(f, "command string is empty"),
            Self::UnknownCommand(raw) => write!(f, "unknown command: {raw}"),
            Self::MissingArgument { command, arg_index } => {
                write!(
                    f,
                    "command `{command}` is missing argument at index {arg_index}"
                )
            }
            Self::InvalidTableId {
                command,
                raw,
                source,
            } => {
                write!(
                    f,
                    "command `{command}`: cannot parse `{raw}` as table ID: {source}"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub enum UserCommand {
    JoinMe(Option<String>),
    JoinTable(TableId, Option<String>),
    Leave,
    SpectateTable(TableId),
    StartGame,
}

impl UserCommand {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        let mut tokens = raw.split_whitespace();
        let command = tokens.next().ok_or(ParseError::EmptyCommand)?;
        let command_args: Vec<&str> = tokens.collect();

        match command {
            "!leave" => Ok(Leave),
            "!joinMe" => {
                let password = command_args.first().map(|s| s.to_string());
                Ok(JoinMe(password))
            }
            "!joinTable" => {
                let raw_id = command_args.get(0).ok_or(ParseError::MissingArgument {
                    command: command.to_string(),
                    arg_index: 0,
                })?;
                let table_id =
                    raw_id
                        .parse::<TableId>()
                        .map_err(|source| ParseError::InvalidTableId {
                            command: command.to_string(),
                            raw: raw_id.to_string(),
                            source,
                        })?;
                let password = command_args.get(1).map(|s| s.to_string());
                Ok(JoinTable(table_id, password))
            }
            "!spectateTable" => {
                let raw_id = command_args.get(0).ok_or(ParseError::MissingArgument {
                    command: command.to_string(),
                    arg_index: 0,
                })?;
                let table_id =
                    raw_id
                        .parse::<TableId>()
                        .map_err(|source| ParseError::InvalidTableId {
                            command: command.to_string(),
                            raw: raw_id.to_string(),
                            source,
                        })?;
                Ok(SpectateTable(table_id))
            }
            "!startGame" => Ok(StartGame),
            _ => Err(ParseError::UnknownCommand(command.to_string())),
        }
    }
}
