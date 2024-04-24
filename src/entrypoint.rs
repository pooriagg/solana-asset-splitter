use {
    solana_program::{
        account_info::AccountInfo,
        entrypoint,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        program_error::ProgramError
    },
    crate::processor::Processor
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    if let Err(_) = Processor::process(program_id, accounts, instruction_data) {
        return Err(
            ProgramError::InvalidInstructionData
        );
    };

    Ok(())
}