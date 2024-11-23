use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use crate::processor::Processor;

// Entry point macro to declare the process_intstruction function as the entrypoint
entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    // Store everything in accounts, all accounts to be read or written to must be passed to the entrypoint function
    // which allows for parallel transactions 
    accounts: &[AccountInfo],
    // Any data passed to the program by caller
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}