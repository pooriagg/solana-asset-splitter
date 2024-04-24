use solana_program::{
    pubkey,
    pubkey::Pubkey
};

pub const PROGRAM_ID: Pubkey = pubkey!("<Pubkey>");

pub fn check_id(program_id: &Pubkey) -> bool {
    *program_id == PROGRAM_ID
}
