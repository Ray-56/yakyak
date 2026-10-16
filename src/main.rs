use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let first = args.next();
    if args.next().is_some() {
        eprintln!("error: invalid argument combination; use --help");
        return ExitCode::from(2);
    }

    match first {
        None => println!("YakYak bootstrap：通信服务尚未实现。"),
        Some(arg) if arg == "--help" => println!(
            "YakYak — bootstrap 工程入口\n\
             通信服务尚未实现；本命令不会启动网络服务。\n\n\
             Usage: yakyak [--help | --version]\n\n\
             Options:\n  --help     显示帮助\n  --version  显示 Cargo 包版本"
        ),
        Some(arg) if arg == "--version" => {
            println!("yakyak {}", env!("CARGO_PKG_VERSION"));
        }
        Some(_) => {
            eprintln!("error: unknown argument; use --help");
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
