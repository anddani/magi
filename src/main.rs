use magi::errors::MagiResult;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() -> MagiResult<()> {
    let workdir = parse_args();
    magi::magi::run(workdir)?;
    Ok(())
}

fn parse_args() -> Option<PathBuf> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut args_iter = args.iter().peekable();

    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-V" | "--version" => {
                println!("magi {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "-C" => {
                let path = args_iter.next().unwrap_or_else(|| {
                    eprintln!("error: option '-C' requires an argument");
                    process::exit(1);
                });
                return Some(PathBuf::from(path));
            }
            arg if arg.starts_with("-C") => {
                // Handle -C<path> (no space)
                return Some(PathBuf::from(&arg[2..]));
            }
            arg => {
                eprintln!("error: unknown option '{}'", arg);
                print_help();
                process::exit(1);
            }
        }
    }

    None
}

fn print_help() {
    println!("magi - A Magit-inspired Git client");
    println!();
    println!("USAGE:");
    println!("    magi [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -C <path>     Run as if started in <path>");
    println!("    -h, --help    Print help information");
    println!("    -V, --version Print version information");
}
