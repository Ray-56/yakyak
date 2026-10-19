use std::ffi::OsString;
use std::process::ExitCode;

fn main() -> ExitCode {
    let result = parse_args(std::env::args_os().skip(1));

    match result {
        Err(ParseError::UnknownArgument) => {
            eprintln!("error: unknown argument; use --help");
            ExitCode::from(2)
        }
        Err(ParseError::TooManyArguments) => {
            eprintln!("error: invalid argument combination; use --help");
            ExitCode::from(2)
        }
        Ok(Command::Bootstrap) => {
            println!("YakYak bootstrap：通信服务尚未实现。");
            ExitCode::SUCCESS
        }
        Ok(Command::Help) => {
            println!(
                "YakYak — bootstrap 工程入口\n\
                通信服务尚未实现；本命令不会启动网络服务。\n\n\
                Usage: yakyak [--help | --version]\n\n\
                Options:\n  --help     显示帮助\n  --version  显示 Cargo 包版本"
            );
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("yakyak {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Bootstrap,
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
enum ParseError {
    TooManyArguments,
    UnknownArgument,
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, ParseError> {
    let mut args = args.into_iter();
    let first = args.next();

    if args.next().is_some() {
        return Err(ParseError::TooManyArguments);
    }

    match first {
        None => Ok(Command::Bootstrap),
        Some(arg) if arg == "--help" => Ok(Command::Help),
        Some(arg) if arg == "--version" => Ok(Command::Version),
        Some(_) => Err(ParseError::UnknownArgument),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_argument_selects_help() {
        let result = parse_args([OsString::from("--help")]);

        assert_eq!(result, Ok(Command::Help));
    }

    #[test]
    fn multiple_arguments_are_rejected() {
        let result = parse_args([OsString::from("--help"), OsString::from("--version")]);

        assert_eq!(result, Err(ParseError::TooManyArguments));
    }
}
