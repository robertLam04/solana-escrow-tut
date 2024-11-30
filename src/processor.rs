// Where the magic happens

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError, 
    msg,
    program_pack::Pack,
    pubkey::Pubkey,
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
        
        msg!("Extracting initializer account...");
        let initializer = next_account_info(account_info_iter)?;
        if !initializer.is_signer {
            msg!("Error: Initializer is not a signer.");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        msg!("Extracting temporary token account...");
        let temp_token_account = next_account_info(account_info_iter)?;
    
        msg!("Extracting token-to-receive account...");
        let token_to_receive_account = next_account_info(account_info_iter)?;
        if *token_to_receive_account.owner != spl_token::id() {
            msg!("Error: Token-to-receive account is not owned by the token program.");
            return Err(ProgramError::IncorrectProgramId);
        }
    
        msg!("Extracting escrow account...");
        let escrow_account = next_account_info(account_info_iter)?;
    
        msg!("Checking rent exemption for escrow account...");
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            msg!("Error: Escrow account is not rent exempt.");
            return Err(EscrowError::NotRentExempt.into());
        }
    
        msg!("Initializing escrow account...");
        let escrow_info = Escrow {
            is_initialized: true,
            initializer_pubkey: *initializer.key,
            temp_token_account_pubkey: *temp_token_account.key,
            initializer_token_to_receive_account_pubkey: *token_to_receive_account.key,
            expected_amount: amount
        };
    
        msg!("Packing escrow account data...");
        
        escrow_info.to_account_info(&escrow_account)?;
    
        msg!("Finding program-derived address (PDA)...");
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
    
        msg!("Extracting token program account...");
        let token_program = next_account_info(account_info_iter)?;
    
        msg!("Creating instruction to change account ownership...");
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;
    
        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;
    
        msg!("Escrow initialization complete.");
        Ok(())
    }    

    fn process_exchange(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Mutable so that we can take elements out of it
        let account_info_iter = &mut accounts.iter();
    
        msg!("Extracting acceptor account...");
        let acceptor = next_account_info(account_info_iter)?;
        if !acceptor.is_signer {
            msg!("Error: Acceptor is not a signer.");
            return Err(ProgramError::MissingRequiredSignature);
        }
    
        msg!("Extracting token-to-send account...");
        let token_to_send_account = next_account_info(account_info_iter)?;
        if *token_to_send_account.owner != spl_token::id() {
            msg!("Error: Token-to-send account is not owned by the token program.");
            return Err(ProgramError::IncorrectProgramId);
        }
    
        msg!("Extracting token-to-receive account...");
        let token_to_receive_account = next_account_info(account_info_iter)?;
        if *token_to_receive_account.owner != spl_token::id() {
            msg!("Error: Token-to-receive account is not owned by the token program.");
            return Err(ProgramError::IncorrectProgramId);
        }
    
        msg!("Extracting PDA's temporary token account...");
        let pda_temp_token_account = next_account_info(account_info_iter)?;
        let pda_temp_token_account_info =
            TokenAccount::unpack(&pda_temp_token_account.try_borrow_data()?)?;
    
        msg!("Finding program-derived address (PDA)...");
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);
    
        msg!("Verifying the amount matches the PDA's temporary token account...");
        if amount != pda_temp_token_account_info.amount {
            msg!("Error: Expected amount does not match the PDA's token account balance.");
            return Err(EscrowError::ExpectedAmountMismatch.into());
        }
    
        msg!("Extracting initializer's main account...");
        let initializer_main_account = next_account_info(account_info_iter)?;
    
        msg!("Extracting initializer's token-to-receive account...");
        let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
    
        msg!("Extracting escrow account...");
        let escrow_account = next_account_info(account_info_iter)?;
        let escrow_state = Escrow::from_account_info(escrow_account)?;
    
        msg!("Validating escrow state...");
        if !escrow_state.is_initialized {
            msg!("Error: Escrow account is not initialized.");
            return Err(ProgramError::UninitializedAccount);
        }
    
        if *initializers_token_to_receive_account.key
            != escrow_state.initializer_token_to_receive_account_pubkey
        {
            msg!("Error: Initializer's token-to-receive account does not match escrow state.");
            return Err(ProgramError::InvalidArgument);
        }
    
        if *pda_temp_token_account.key != escrow_state.temp_token_account_pubkey {
            msg!("Error: PDA's temp token account does not match escrow state.");
            return Err(ProgramError::InvalidArgument);
        }
    
        if *initializer_main_account.key != escrow_state.initializer_pubkey {
            msg!("Error: Initializer's main account does not match escrow state.");
            return Err(ProgramError::InvalidArgument);
        }
    
        msg!("Extracting token program account...");
        let token_program = next_account_info(account_info_iter)?;
    
        msg!("Transferring tokens to the initializer...");
        let transfer_to_initializer_ix = spl_token::instruction::transfer(
            token_program.key,
            token_to_send_account.key,
            initializers_token_to_receive_account.key,
            acceptor.key,
            &[&acceptor.key],
            escrow_state.expected_amount,
        )?;
        invoke(
            &transfer_to_initializer_ix,
            &[
                token_to_send_account.clone(),
                initializers_token_to_receive_account.clone(),
                acceptor.clone(),
                token_program.clone(),
            ],
        )?;
    
        msg!("Extracting PDA account...");
        let pda_account = next_account_info(account_info_iter)?;
    
        msg!("Transferring tokens to the taker...");
        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pda_temp_token_account.key,
            token_to_receive_account.key,
            &pda,
            &[&pda],
            pda_temp_token_account_info.amount,
        )?;
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pda_temp_token_account.clone(),
                token_to_receive_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;
    
        msg!("Closing PDA's temporary token account...");
        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pda_temp_token_account.key,
            initializer_main_account.key,
            &pda,
            &[&pda],
        )?;
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
        **initializer_main_account.lamports.borrow_mut() = initializer_main_account
            .lamports()
            .checked_add(escrow_account.lamports())
            .ok_or(EscrowError::AmountOverflow)?;
        **escrow_account.lamports.borrow_mut() = 0;
        *escrow_account.try_borrow_mut_data()? = &mut [];
    
        msg!("Exchange process complete.");
        Ok(())
    }
    

}