#[derive(Debug, Clone, clap::Parser)]
#[command(version, about, long_about = None, subcommand_required = true, arg_required_else_help = true)]
pub enum Cli {
    #[command(about = "Pack files into a BOF archive.")]
    Pack {
        #[arg(help = "Input file(-s)", required = true, num_args=1..)]
        files: Vec<std::path::PathBuf>,
        #[arg(help = "Output file", short, long, required = true)]
        output: std::path::PathBuf,
    },
    #[command(about = "Unpack a BOF archive.")]
    Extract {
        #[arg(help = "Input file (BOF archive)", required = true)]
        archive: std::path::PathBuf,
        #[arg(help = "Output directory", short, long, default_value = ".")]
        output: std::path::PathBuf,
    },
    #[command(about = "List files in a BOF archive.")]
    List {
        #[arg(help = "Input file (BOF archive)", required = true)]
        archive: std::path::PathBuf,
    },
}
