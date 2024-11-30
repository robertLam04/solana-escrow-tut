// Define state objects used by the processor here and serialize/deserialize objects into arrays of u8

use solana_program::{
    pubkey::Pubkey,
    program_error::ProgramError,
    msg
};
use borsh::{BorshDeserialize, BorshSerialize};

// Save pub keys here so that a user knows what accounts to pass into the entrypoint
// Save initializer_token_to_receive_account pubkey so the escrow know where to send acceptors tokens
// Also to ensure that the acceptor passes the same temp account that was initialized
// It's the programs responsibility to chck that recieved accounts == expected accounts
// Save expected amount to ensure acceptor sends enough tokens
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct Escrow {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub temp_token_account_pubkey: Pubkey,
    pub initializer_token_to_receive_account_pubkey: Pubkey,
    pub expected_amount: u64,
}

impl Escrow {

    pub const LEN: usize = 1 + 32 + 32 + 32 + 8;

    pub fn from_account_info(account_info: &solana_program::account_info::AccountInfo) -> Result<Self, ProgramError> {
        let data = &account_info.data.borrow();
        Escrow::try_from_slice(data).map_err(|_| ProgramError::InvalidAccountData)
    }

    /// Serialize the Escrow struct into the account data
    pub fn to_account_info(&self, account_info: &solana_program::account_info::AccountInfo) -> Result<(), ProgramError> {
        let mut data = account_info.data.borrow_mut();
        msg!("Account data buffer size: {}", data.len());
        msg!("Expected data size: {}", Escrow::LEN);
        self.serialize(&mut *data).map_err(|_| ProgramError::AccountDataTooSmall)
    }

}
