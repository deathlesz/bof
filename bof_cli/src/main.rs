use std::path::PathBuf;

use bof::BofArchive;

mod cli;

fn main() {
    let matches = cli::get_matches();

    match matches.subcommand() {
        Some(("pack", pack_matches)) => {
            let files = pack_matches
                .get_many::<PathBuf>("file")
                .unwrap()
                .collect::<Vec<&PathBuf>>();

            let output = pack_matches.get_one::<PathBuf>("output").unwrap();

            pack_files(files, output).unwrap_or_else(eprintln_and_exit);
        }
        Some(("unpack", unpack_matches)) => {
            let archive = unpack_matches
                .get_one::<PathBuf>("file")
                .expect("must be present");

            let output = unpack_matches.get_one::<PathBuf>("output");

            unpack_files(archive, output).unwrap_or_else(eprintln_and_exit);
        }
        _ => unreachable!(),
    };
}

fn pack_files(files: Vec<&PathBuf>, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = BofArchive::new();

    for file in files {
        archive.add(
            file.file_name().unwrap().to_str().unwrap().to_string(),
            std::fs::read(file)?,
        );
    }

    Ok(std::fs::write(output, archive.build())?)
}

fn unpack_files(
    archive: &PathBuf,
    output: Option<&PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let contents = std::fs::read(archive)?;

    let archive = BofArchive::try_from(contents.as_slice())?;
    let output = output
        .cloned()
        .unwrap_or_else(|| std::env::current_dir().expect("failed to get current directory"));
    for file in archive.files() {
        std::fs::write(output.join(file.filename()), file.contents())?;
    }

    Ok(())
}

fn eprintln_and_exit(error: Box<dyn std::error::Error>) {
    eprintln!("{}", error);
    std::process::exit(1);
}
