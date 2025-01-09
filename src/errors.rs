use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Arithmetic error!")]
    ArithmeticError,
    #[error("amount entered in redemption is not valid!")]
    InvalidRedemptionAmount
}

impl From<CustomError> for ProgramError {
    fn from(value: CustomError) -> Self {
        ProgramError::Custom(value as u32)
    }
}