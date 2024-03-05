use crate::error::BofError;

pub mod error;

const MAJOR_VERSION: u8 = 0;
const MINOR_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchivedFile {
    filename: String,
    contents: Vec<u8>,
}

impl ArchivedFile {
    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents
    }
}

impl ArchivedFile {
    pub fn new(filename: String, contents: Vec<u8>) -> Self {
        Self { filename, contents }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct BofArchive {
    files: Vec<ArchivedFile>,
}

impl BofArchive {
    pub fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add(&mut self, filename: String, contents: Vec<u8>) {
        self.files.push(ArchivedFile::new(filename, contents));
    }

    pub fn files(&self) -> &[ArchivedFile] {
        self.files.as_slice()
    }

    pub fn build(self) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::with_capacity(10);

        // signature (BOF) + major version + minor version + reserved byte
        output.extend([66, 79, 70, MAJOR_VERSION, MINOR_VERSION, 0]);

        for file in self.files {
            output.extend_from_slice((file.filename + "\0").as_bytes());
            output.extend((file.contents.len() as u64).to_le_bytes());
            output.extend(&file.contents);
            output.extend(crc32fast::hash(&file.contents).to_le_bytes());
        }

        output.extend(crc32fast::hash(&output).to_le_bytes());

        output
    }

    fn parse(bytes: &[u8]) -> Result<Self, BofError> {
        if bytes.len() < 10 {
            return Err(BofError::InvalidArchive("file is too small".into()));
        }

        if &bytes[..3] != b"BOF" {
            return Err(BofError::InvalidArchive("signature is not BOF".into()));
        }

        let (major, _) = (u8::from_le_bytes([bytes[3]]), u8::from_le_bytes([bytes[4]]));

        if major > MAJOR_VERSION {
            return Err(BofError::VersionTooHigh {
                expected: MAJOR_VERSION,
                version: major,
            });
        }

        if crc32fast::hash(&bytes[..(bytes.len() - 4)]).to_le_bytes() != bytes[bytes.len() - 4..] {
            return Err(BofError::InvalidChecksum);
        }

        // skipping reserved byte at offset 5
        let mut offset = 6;
        let mut archive = BofArchive::new();

        while bytes[offset..].len() > 4 {
            // collect all bytes in a string until we meet 0x0 byte
            let file_name = bytes[offset..bytes.len() - 3]
                .iter()
                .take_while(|byte| **byte != 0x0)
                .map(|byte| *byte as char)
                .collect::<String>();

            let file_size_offset = offset + file_name.len() + 1;
            let file_size_bytes = bytes
                .get(file_size_offset..file_size_offset + 8)
                .ok_or_else(|| {
                    BofError::InvalidArchive("unexpected EOF while parsing file size".into())
                })?;
            let file_size = u64::from_le_bytes(file_size_bytes.try_into().unwrap());

            let file_contents_offset = file_size_offset + 8;
            let file_contents = bytes
                .get(file_contents_offset..(file_contents_offset + file_size as usize))
                .ok_or_else(|| {
                    BofError::InvalidArchive("unexpected EOF while parsing file contents".into())
                })?;

            let checksum_offset = file_contents_offset + file_size as usize;
            let checksum_bytes =
                bytes
                    .get(checksum_offset..checksum_offset + 4)
                    .ok_or_else(|| {
                        BofError::InvalidArchive("unexpected EOF while parsing checksum".into())
                    })?;
            let checksum = u32::from_le_bytes(checksum_bytes.try_into().unwrap());

            if checksum != crc32fast::hash(file_contents) {
                return Err(BofError::InvalidChecksum);
            }

            offset = checksum_offset + 4;
            archive.add(file_name, file_contents.to_vec());
        }

        Ok(archive)
    }
}

impl<'a> TryFrom<&'a [u8]> for BofArchive {
    type Error = BofError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}
