use solana_program::pubkey::Pubkey;

#[derive(Debug)]
pub enum Event {
    LamportsSplitted {
        source_account: Pubkey,
        destination_accounts: Vec<Pubkey>,
        amounts: Vec<u64>
    },
    SplTokensSplittedFromSignleMint {
        operator: Pubkey,
        source_account: Pubkey,
        destination_accounts: Vec<Pubkey>,
        amounts: Vec<u64>
    },
    SplTokensSplittedFromMultipleMints {
        operator: Pubkey,
        source_accounts: Vec<Pubkey>,
        destination_accounts: Vec<Pubkey>,
        amounts: Vec<u64>
    }
}