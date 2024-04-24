#![allow(warnings)]

use {
    borsh::{
        BorshDeserialize,
        BorshSerialize
    },
    solana_program::{
        program_error::ProgramError,
        hash::hash
    }
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SplitLamports {
    pub amounts: Vec<u64>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SplitSplTokensFromSingleMint {
    pub amounts: Vec<u64>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SplitSplTokensFromMultipleMints {
    pub amounts: Vec<u64>,
    pub m: u16
}

#[derive(Debug, PartialEq)]
pub enum SplitterInstruction {
    /// Splits specified lamports to the desired addresses
    /// 
    /// Amounts must be in lamport
    /// 
    /// Accounts expected by this instruction:
    /// 
    ///     0. `[writable,signer]` system-program owned account as source account
    ///     1. `[]` system-program
    ///     2. 2..2+M `[writable]` M system-program owned accounts as destination accounts
    SplitLamports(Vec<u64>),
    /// Splits specified spl tokens to the desired token accounts
    /// 
    /// Amoutns must be raw amounts
    /// 
    /// Accounts expected by this instruction:
    /// 
    ///     0. `[signer]` owner/delegator of the source token account
    ///     1. `[]` token-standard-program
    ///     2. `[writable]` source token account
    ///     3. 3..3+M `[writable]` M destination token accounts
    SplitSplTokensFromSingleMint(Vec<u64>),
    /// Splits specified spl tokens to the desired token accounts
    /// 
    /// Amoutns must be raw amounts
    /// 
    /// Accounts expected by this instruction:
    /// 
    ///     * This instruction requires a `m` parameter to be passed as data(u16)
    ///     0. `[signer]` owner/delegator of the source token account
    ///     1. `[]` token-standard-program
    ///     2. 2..2+m `[writable]` m source accounts
    ///     3. m+2.. `[wrtiable]` m destination accounts
    SplitSplTokensFromMultipleMints(
        Vec<u64>,
        u16
    )
}

impl SplitterInstruction {
    pub fn unpack(ix_data: &[u8]) -> Result<Self, ProgramError> {
        if ix_data.len() <= 8 {
            return Err(
                ProgramError::InvalidInstructionData
            );
        };

        let (
            ix_splitlamports,
            ix_splitspltokensfromsinglemint,
            ix_splitspltokensfrommultiplemints 
        ) = (
            Self::get_discriminator("instruction:splitlamports"),
            Self::get_discriminator("instruction:splitspltokensfromsinglemint"),
            Self::get_discriminator("instruction:splitspltokensfrommultiplemints")
        );

        let ix_identifier: [u8; 8] = ix_data[..8].try_into().unwrap();
        if ix_identifier == ix_splitlamports {
            let data = &ix_data[8..];
            let ix = SplitLamports::try_from_slice(&data).unwrap();

            return Ok(
                Self::SplitLamports(ix.amounts)
            );
        } else if ix_identifier == ix_splitspltokensfromsinglemint {
            let data = &ix_data[8..];
            let ix = SplitSplTokensFromSingleMint::try_from_slice(&data).unwrap();

            return Ok(
                Self::SplitSplTokensFromSingleMint(ix.amounts)
            );
        } else if ix_identifier == ix_splitspltokensfrommultiplemints {
            let data = &ix_data[8..];
            let ix = SplitSplTokensFromMultipleMints::try_from_slice(&data).unwrap();

            return Ok(
                Self::SplitSplTokensFromMultipleMints(
                    ix.amounts,
                    ix.m
                )
            );
        } else {
            return Err(
                ProgramError::InvalidInstructionData
            );
        };
    }

    pub fn get_discriminator(dis: &str) -> [u8; 8] {
        let sha256 = hash(
            dis.as_bytes().as_ref()
        );
    
        let dis: [u8; 8] = sha256.as_ref()[..8].try_into().unwrap();
        dis
    }
}
