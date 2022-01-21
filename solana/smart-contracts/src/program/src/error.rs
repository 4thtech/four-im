use {
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

/// Errors that may be returned by the program
#[derive(Debug, Error)]
pub enum InstantMessagingError {
    /// Incorrect account address derivation
    #[error("Incorrect account address derivation")]
    AddressDerivationMismatch,
}

impl From<DocumentsError> for ProgramError {
    fn from(e: DocumentsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for InstantMessagingError {
    fn type_of() -> &'static str {
        "Instant Messaging Error"
    }
}