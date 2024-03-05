#[derive(Debug, Clone, PartialEq, Hash, thiserror::Error)]
pub enum BofError {
    #[error("invalid archive: {0}")]
    InvalidArchive(String),
    #[error("archive version is too high: {expected} < {version}")]
    VersionTooHigh { expected: u8, version: u8 },
    #[error("invalid checksum")]
    InvalidChecksum,
}
