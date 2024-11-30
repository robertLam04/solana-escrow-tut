use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum EscrowError {
    // Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction = 0x01,
    #[error("Not Rent Exempt")]
    NotRentExempt = 0x02,
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch = 0x03,
    #[error("Overflow")]
    AmountOverflow = 0x04
}

// Implement the generic From trait which the ? operator wants to use
impl From<EscrowError> for ProgramError {
    fn from (e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}