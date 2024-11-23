// Where the magic happens

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::invoke
};

use crate::{instruction::EscrowInstruction, error::EscrowError, state::Escrow};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        // Pass the reference to the slice holding instruction data from the entrypoint into the unpack function
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            }
        }
    }

    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Mutable so that we can take elements out of it
        let account_info_iter = &mut accounts.iter();
        // First account we expect (defines in instruction.rs) escrow initializer
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_token_account = next_account_info(account_info_iter)?;

        let token_to_receive_account = next_account_info(account_info_iter)?;
        // Check if the receiving account is owned by the token program
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // Get the next account
        let escrow_account = next_account_info(account_info_iter)?;
        // Get the rent account which is the next expected account 
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        // Check if rent is exempt based on it's balance and size
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        // Accessing the data field with unpack_unchecked default function from Pack
        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.try_borrow_data()?)?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
        escrow_info.expected_amount = amount;

        // default function that calls our pack_into_slice function
        Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;

        // Create pda by passing in an arrray of seeds and program id
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
        
        // Get the token program since it is used as second argument invoke (CPI)
        let token_program = next_account_info(account_info_iter)?;
        // Create the instruction that we want the token program to execute, specifically set_authority
        // This uses signature extension:
        //     - Since the initializer signed the init escrow transaction, the program can extend the signature to the CPI
        // This instruction builder function also checks that token_program is actually the account of the token program
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        
        // Call the instruction and pass accounts needed (can view the needed accounts by going to the token programs instruction.rs)
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

}