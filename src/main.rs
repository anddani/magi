use magi::errors::MagiResult;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() -> MagiResult<()> {
    let (workdir, language) = parse_args();
    magi::magi::run(workdir, language)?;
    Ok(())
}

fn parse_args() -> (Option<PathBuf>, Option<String>) {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut args_iter = args.iter();
    let mut workdir: Option<PathBuf> = None;
    let mut language: Option<String> = None;

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
                workdir = Some(PathBuf::from(path));
            }
            arg if arg.starts_with("-C") => {
                workdir = Some(PathBuf::from(&arg[2..]));
            }
            arg if arg.starts_with("--language=") => {
                language = Some(arg["--language=".len()..].to_string());
            }
            "--language" => {
                let lang = args_iter.next().unwrap_or_else(|| {
                    eprintln!("error: option '--language' requires an argument");
                    process::exit(1);
                });
                language = Some(lang.to_string());
            }
            arg if arg.starts_with('-') => {
                eprintln!("error: unknown option '{}'", arg);
                print_help();
                process::exit(1);
            }
            arg => {
                workdir = Some(PathBuf::from(arg));
            }
        }
    }

    (workdir, language)
}

fn print_help() {
    println!("magi - A Magit-inspired Git client");
    println!();
    println!("USAGE:");
    println!("    magi [OPTIONS] [path]");
    println!();
    println!("ARGS:");
    println!("    [path]                      Run as if started in <path>");
    println!();
    println!("OPTIONS:");
    println!("    -C <path>                   Run as if started in <path>");
    println!("    --language <lang>           Set UI language (en, sv)");
    println!("    -h, --help                  Print help information");
    println!("    -V, --version               Print version information");
}
