#![allow(warnings)]

use {
    solana_program::{
        account_info::{
            AccountInfo,
            next_account_info
        },
        pubkey::Pubkey,
        entrypoint::ProgramResult,
        msg,
        system_instruction::transfer as transfer_lamports,
        program::invoke,
        program_error::ProgramError
    },
    spl_token::{
        instruction::transfer as transfer_spl_tokens,
        ID as token_program_id
    },
    crate::{
        instruction::SplitterInstruction,
        event::Event,
        error::SplitterError
    }
};

pub struct Processor {}
impl Processor {
    pub fn process_split_lamports(
        accounts: &[AccountInfo],
        amounts: &Vec<u64>
    ) -> ProgramResult {
        let accounts_info = &mut accounts.iter();

        let source_account = next_account_info(accounts_info)?;
        let system_program_info = next_account_info(accounts_info)?;

        for amount in amounts.iter() {
            let destination_account = next_account_info(accounts_info)?;

            invoke(
                &transfer_lamports(
                    &source_account.key,
                    &destination_account.key,
                    *amount
                ),
                &[
                    source_account.clone(),
                    destination_account.clone(),
                    system_program_info.clone()
                ]
            )?;
        };

        let account_keys = accounts[1..]
            .iter()
            .map(|account_data| *account_data.key)
            .collect::<Vec<_>>();

        let event = Event::LamportsSplitted {
            source_account: *source_account.key,
            destination_accounts: account_keys,
            amounts: (*amounts).to_vec()
        };
        msg!("Event: {:?}", event);

        Ok(())
    }

    pub fn process_split_spl_tokens_from_single_mint(
        accounts: &[AccountInfo],
        amounts: &Vec<u64>
    ) -> ProgramResult {
        let accounts_info = &mut accounts.iter();

        let operator = next_account_info(accounts_info)?;
        let token_program = next_account_info(accounts_info)?;
        let source_ata = next_account_info(accounts_info)?;

        for amount in amounts.iter() {
            let destination_ata = next_account_info(accounts_info)?;

            invoke(
                &transfer_spl_tokens(
                    &token_program.key,
                    &source_ata.key,
                    &destination_ata.key,
                    &operator.key,
                    &[],
                    *amount
                )?,
                &[
                    source_ata.clone(),
                    destination_ata.clone(),
                    operator.clone()
                ]
            )?;
        };

        let accounts_keys = accounts[2..]
            .iter()
            .map(|account_data| *account_data.key)
            .collect::<Vec<_>>();

        let event = Event::SplTokensSplittedFromSignleMint {
            operator: *operator.key,
            source_account: *source_ata.key,
            destination_accounts: accounts_keys,
            amounts: (*amounts).to_vec()
        };
        msg!("Event: {:?}", event);

        Ok(())
    }

    pub fn process_split_spl_tokens_from_multiple_mints(
        accounts: &[AccountInfo],
        amounts: &Vec<u64>,
        m: &u16
    ) -> ProgramResult {
        let operator = &accounts[0];
        let token_program = &accounts[1];

        let expected_accounts_len = ((m * 2) + 2) as usize;
        if accounts.len() != expected_accounts_len {
            return Err(
                ProgramError::Custom(
                    SplitterError::InvalidMParameter(
                        *m,
                        expected_accounts_len,
                        accounts.len()
                    ).into()
                )
            );
        };

        let source_atas = accounts
            .get(2..=((*m + 1) as usize))
            .unwrap();
        let destination_atas = accounts
            .get((m + 2) as usize..)
            .unwrap();

        let mut count = 0usize;
        for amount in amounts.iter() {
            let source_ata = source_atas
                .get(count)
                .unwrap();
            let destination_ata = destination_atas
                .get(count)
                .unwrap();

            invoke(
                &transfer_spl_tokens(
                    &token_program.key,
                    &source_ata.key,
                    &destination_ata.key,
                    &operator.key,
                    &[],
                    *amount
                )?,
                &[
                    source_ata.clone(),
                    destination_ata.clone(),
                    operator.clone()
                ]
            )?;

            count = count
                .checked_add(1usize)
                .unwrap();
        };

        let source_atas = source_atas
            .iter()
            .map(|ata_info| *ata_info.key)
            .collect::<Vec<_>>();
        let destination_atas = destination_atas
            .iter()
            .map(|ata_info| *ata_info.key)
            .collect::<Vec<_>>();

        let event = Event::SplTokensSplittedFromMultipleMints {
            operator: *operator.key,
            source_accounts: source_atas,
            destination_accounts: destination_atas,
            amounts: (*amounts).to_vec()
        };
        msg!("Event: {:?}", event);

        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8]
    ) -> ProgramResult {
        let instruction = SplitterInstruction::unpack(instruction_data).unwrap();

        match instruction {
            SplitterInstruction::SplitLamports(amounts) => {
                msg!("Instruction: SplitLamports");
                Self::process_split_lamports(
                    accounts,
                    &amounts
                ).unwrap();
            },
            SplitterInstruction::SplitSplTokensFromSingleMint(amounts) => {
                msg!("Instruction: SplitSplTokensFromSingleMint");
                Self::process_split_spl_tokens_from_single_mint(
                    accounts,
                    &amounts
                ).unwrap();
            },
            SplitterInstruction::SplitSplTokensFromMultipleMints(
                amounts,
                m
            ) => {
                msg!("Instruction: SplitSplTokensFromMultipleMints");
                Self::process_split_spl_tokens_from_multiple_mints(
                    accounts,
                    &amounts,
                    &m
                ).unwrap();
            }
        };

        Ok(())
    }
}

mod helper {
    //
}