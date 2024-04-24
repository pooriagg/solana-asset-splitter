use solana_program::{
    pubkey,
    pubkey::Pubkey
};

pub const PROGRAM_ID: Pubkey = pubkey!("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");

pub fn check_id(program_id: &Pubkey) -> bool {
    *program_id == PROGRAM_ID
}