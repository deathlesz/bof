use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::{eyre, Result};

use bof::BofArchive;
use cli::Cli;

mod cli;

fn main() -> Result<()> {
    let matches = Cli::parse();

    match matches {
        Cli::Pack { files, output } => pack_files(files, output),
        Cli::Extract { archive, output } => unpack_files(archive, output),
        Cli::List { archive } => list_files(archive),
    }
}

fn pack_files(files: Vec<PathBuf>, output: PathBuf) -> Result<()> {
    let mut archive = BofArchive::new();

    for file in files {
        archive.add(
            file.file_name()
                .ok_or(eyre!("couldn't identify filename"))?
                .to_str()
                .ok_or(eyre!("filename is not valid unicode"))?
                .to_string(),
            std::fs::read(file)?,
        );
    }

    Ok(std::fs::write(output, archive.build())?)
}

fn unpack_files(archive: PathBuf, output: PathBuf) -> Result<()> {
    let contents = std::fs::read(archive)?;

    let archive = BofArchive::try_from(contents.as_slice())?;
    for file in archive.files() {
        std::fs::write(output.join(file.filename()), file.contents())?;
    }

    Ok(())
}

fn list_files(archive: PathBuf) -> Result<()> {
    let contents = std::fs::read(archive)?;

    let archive = BofArchive::try_from(contents.as_slice())?;
    let mut stdout = std::io::stdout().lock();

    for file in archive.files() {
        writeln!(stdout, "{}\t{}B", file.filename(), file.contents().len())?;
    }

    Ok(())
}
