// Where the magic happens

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError, 
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::{invoke, invoke_signed}
};

use spl_token::state::Account as TokenAccount;
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
            },
            EscrowInstruction::Exchange { amount } => {
                msg!("Instruction: Exchange");
                Self::process_exchange(accounts, amount, program_id)
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

    fn process_exchange(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey
    ) -> ProgramResult {

        // Mutable so that we can take elements out of it
        let account_info_iter = &mut accounts.iter();
        // First account we expect (defined in instruction.rs) acceptor
        let acceptor = next_account_info(account_info_iter)?;

        if !acceptor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let token_to_send_account = next_account_info(account_info_iter)?;
        if *token_to_send_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_to_receive_account = next_account_info(account_info_iter)?;
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let pda_temp_token_account = next_account_info(account_info_iter)?;
        // Unpack the pda temp token account data into a TokenAccount struct
        let pda_temp_token_account_info = TokenAccount::unpack(&pda_temp_token_account.try_borrow_data()?)?;

        // Derive the PDA
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        if amount != pda_temp_token_account_info.amount {
            return Err(EscrowError::ExpectedAmountMismatch.into());
        }

        let initializer_main_account = next_account_info(account_info_iter)?;
        let initializers_token_to_receive_account = next_account_info(account_info_iter)?;  
        let escrow_account = next_account_info(account_info_iter)?;
        let escrow_state = Escrow::unpack(&escrow_account.try_borrow_mut_data()?)?;
        
        // Check if escrow is initialized and if state mathes inputted accounts
        if !escrow_state.is_initialized {
            return Err(ProgramError::UninitializedAccount);
        }

        if *initializers_token_to_receive_account.key != escrow_state.initializer_token_to_receive_account_pubkey {
            return Err(ProgramError::InvalidArgument);
        }

        if *pda_temp_token_account.key != escrow_state.temp_token_account_pubkey {
            return Err(ProgramError::InvalidArgument);
        }

        if *initializer_main_account.key != escrow_state.initializer_pubkey {
            return Err(ProgramError::InvalidArgument)
        }

        let token_program = next_account_info(account_info_iter)?;

        let transfer_to_initializer_ix = spl_token::instruction::transfer(
            token_program.key,
            token_to_send_account.key,
            initializers_token_to_receive_account.key,
            acceptor.key,
            &[&acceptor.key],
            escrow_state.expected_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                token_to_send_account.clone(),
                initializers_token_to_receive_account.clone(),
                acceptor.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            // Token program id
            token_program.key,
            // Source token account
            pda_temp_token_account.key,
            // Dest token account
            token_to_receive_account.key,
            // Authority (authorized to sign)
            &pda,
            // Signers
            &[&pda],
            //Amount
            pda_temp_token_account_info.amount,
        )?;

        msg!("Calling the token program to transfer tokens to the taker...");

        invoke_signed(
            // Instruction
            &transfer_to_taker_ix,
            // Accounts involved
            &[
                pda_temp_token_account.clone(), // Source token acc
                token_to_receive_account.clone(), // Dest token acc
                pda_account.clone(), // PDA (authority)
                token_program.clone(), // Spl token program
            ],
            &[&[&b"escrow"[..], &[bump_seed]]], // Signer seeds for the PDA
        )?;

        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pda_temp_token_account.key,
            initializer_main_account.key,
            &pda,
            &[&pda]
        )?;
        msg!("Calling the token program to close pda's temp account...");
        invoke_signed(
            &close_pdas_temp_acc_ix,
            &[
                pda_temp_token_account.clone(),
                initializer_main_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        msg!("Closing the escrow account...");
        // Adds escrow account balance to intializers account (this is allowed since the program is the owner of escrow_account)
        **initializer_main_account.lamports.borrow_mut() = initializer_main_account.lamports()
            .checked_add(escrow_account.lamports())
            .ok_or(EscrowError::AmountOverflow)?;
        **escrow_account.lamports.borrow_mut() = 0;
        // Remember to clear the data field so that the account is deleted
        *escrow_account.try_borrow_mut_data()? = &mut [];

        Ok(())

    }

}