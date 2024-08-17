use std::collections::BTreeSet;

pub use crate::error::Error;

mod error;

const MAJOR_VERSION: u8 = 0;
const MINOR_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    files: BTreeSet<ArchivedFile>,
}

impl BofArchive {
    pub fn new() -> Self {
        Self {
            files: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, filename: String, contents: Vec<u8>) {
        self.files.insert(ArchivedFile::new(filename, contents));
    }

    pub fn remove(&mut self, filename: &str) {
        self.files.retain(|file| file.filename != filename);
    }

    pub fn files(&self) -> impl Iterator<Item = &ArchivedFile> {
        self.files.iter()
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

    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 10 {
            return Err(Error::InvalidArchive("file is too small".into()));
        }

        if &bytes[..3] != b"BOF" {
            return Err(Error::InvalidArchive("signature is not BOF".into()));
        }

        let (major, _) = (u8::from_le_bytes([bytes[3]]), u8::from_le_bytes([bytes[4]]));

        if major > MAJOR_VERSION {
            return Err(Error::VersionTooHigh {
                expected: MAJOR_VERSION,
                version: major,
            });
        }

        if crc32fast::hash(&bytes[..(bytes.len() - 4)]).to_le_bytes() != bytes[bytes.len() - 4..] {
            return Err(Error::InvalidChecksum);
        }

        // skipping reserved byte at offset 5
        let mut offset = 6;
        let mut archive = BofArchive::new();

        while bytes[offset..].len() > 4 {
            let file_name = Self::parse_file_name(&bytes[offset..bytes.len() - 3]);

            let file_size_offset = offset + file_name.len() + 1;
            let file_size_bytes = bytes
                .get(file_size_offset..file_size_offset + 8)
                .ok_or_else(|| {
                    Error::InvalidArchive("unexpected EOF while parsing file size".into())
                })?;
            let file_size = u64::from_le_bytes(file_size_bytes.try_into().unwrap());

            let file_contents_offset = file_size_offset + 8;
            let file_contents = bytes
                .get(file_contents_offset..(file_contents_offset + file_size as usize))
                .ok_or_else(|| {
                    Error::InvalidArchive("unexpected EOF while parsing file contents".into())
                })?;

            let checksum_offset = file_contents_offset + file_size as usize;
            let checksum_bytes =
                bytes
                    .get(checksum_offset..checksum_offset + 4)
                    .ok_or_else(|| {
                        Error::InvalidArchive("unexpected EOF while parsing checksum".into())
                    })?;
            let checksum = u32::from_le_bytes(checksum_bytes.try_into().unwrap());

            if checksum != crc32fast::hash(file_contents) {
                return Err(Error::InvalidChecksum);
            }

            offset = checksum_offset + 4;
            archive.add(file_name, file_contents.to_vec());
        }

        Ok(archive)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn parse_unchecked(bytes: &[u8]) -> Self {
        // skipping reserved byte at offset 5
        let mut offset = 6;
        let mut archive = BofArchive::new();

        while bytes[offset..].len() > 4 {
            let file_name = Self::parse_file_name(&bytes[offset..bytes.len() - 3]);

            let file_size_offset = offset + file_name.len() + 1;
            let file_size_bytes = bytes.get(file_size_offset..file_size_offset + 8).unwrap();
            let file_size = u64::from_le_bytes(file_size_bytes.try_into().unwrap());

            let file_contents_offset = file_size_offset + 8;
            let file_contents = bytes
                .get(file_contents_offset..(file_contents_offset + file_size as usize))
                .unwrap();

            offset = file_contents_offset + file_size as usize + 4;
            archive.add(file_name, file_contents.to_vec());
        }

        archive
    }

    fn parse_file_name(bytes: &[u8]) -> String {
        // collect all bytes in a string until we meet 0x0 byte
        bytes
            .iter()
            .take_while(|byte| **byte != 0x0)
            .map(|byte| *byte as char)
            .collect::<String>()
    }
}

impl<'a> TryFrom<&'a [u8]> for BofArchive {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::parse(bytes)
    }
}

impl TryFrom<Vec<u8>> for BofArchive {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}

impl<const N: usize> TryFrom<[u8; N]> for BofArchive {
    type Error = Error;

    fn try_from(bytes: [u8; N]) -> Result<Self, Self::Error> {
        Self::parse(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_archive() -> BofArchive {
        let mut archive = BofArchive::new();

        archive.add("file1".into(), vec![0xBE, 0xEF]);
        archive.add("file2".into(), vec![0xDE, 0xAD, 0x00, 0x71]);
        archive.add("file3".into(), vec![0x36, 0x39, 0x34, 0x30, 0x32]);

        archive
    }

    #[test]
    fn creation() {
        let archive = create_archive();

        assert_eq!(archive.files().count(), 3);

        let bytes = archive.build();

        // the smallest size is 10 bytes
        assert!(bytes.len() >= 10);

        assert_eq!(&bytes[..3], b"BOF");
        assert_eq!(&bytes[3..5], [MAJOR_VERSION, MINOR_VERSION]);
        assert_eq!(bytes[5], 0);

        assert_eq!(&bytes[6..12], "file1\0".as_bytes());
        assert_eq!(&bytes[12..20], 2u64.to_le_bytes());
        assert_eq!(&bytes[20..22], [0xBE, 0xEF]);
        assert_eq!(&bytes[22..26], [0x20, 0x6E, 0x2A, 0x0B]);

        assert_eq!(&bytes[26..32], "file2\0".as_bytes());
        assert_eq!(&bytes[32..40], 4u64.to_le_bytes());
        assert_eq!(&bytes[40..44], [0xDE, 0xAD, 0x00, 0x71]);
        assert_eq!(&bytes[44..48], [0x2F, 0x9E, 0x6D, 0x11]);

        assert_eq!(&bytes[48..54], "file3\0".as_bytes());
        assert_eq!(&bytes[54..62], 5u64.to_le_bytes());
        assert_eq!(&bytes[62..67], [0x36, 0x39, 0x34, 0x30, 0x32]);
        assert_eq!(&bytes[67..71], [0x2F, 0x27, 0x93, 0x51]);

        assert_eq!(bytes[bytes.len() - 4..], [0x70, 0x2F, 0x6B, 0xBF]);
    }

    #[test]
    fn can_parse_self_generated() {
        let archive = create_archive();
        let bytes = archive.clone().build();

        assert!(BofArchive::try_from(bytes.clone()).is_ok());
        assert_eq!(unsafe { BofArchive::parse_unchecked(&bytes) }, archive);
    }
}
