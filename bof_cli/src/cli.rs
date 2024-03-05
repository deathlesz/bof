use clap::{Arg, ArgAction, ArgMatches, Command};

pub fn get_matches() -> ArgMatches {
    Command::new("bof")
        .about("A CLI tool for (un-)packing BOF archives.")
        .version("0.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("pack")
                .alias("p")
                .about("Pack a bunch of files (pun intended) into a BOF archive.")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file")
                        .required(true)
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                )
                .arg(
                    Arg::new("file")
                        .help("Input file(-s)")
                        .required(true)
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .action(ArgAction::Set)
                        .num_args(1..),
                ),
        )
        .subcommand(
            Command::new("extract")
                .alias("x")
                .about("Unpack a BOF archive.")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file")
                        .required(true)
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                )
                .arg(
                    Arg::new("file")
                        .help("Input file (BOF archive)")
                        .required(true)
                        .value_parser(clap::value_parser!(std::path::PathBuf))
                        .action(ArgAction::Set),
                ),
        )
        .get_matches()
}
