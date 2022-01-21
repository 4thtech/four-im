use {
    instant_messaging::{
        id,
        process_instruction,
    },
    solana_program::{
        pubkey::Pubkey,
    },
    solana_program_test::*,
};

pub fn program_test() -> ProgramTest {
    let mut program_test = ProgramTest::new(
        "instant_messaging",
        id(),
        processor!(process_instruction),
    );

    // Dial down the BPF compute budget to detect if the program gets bloated in the future
    program_test.set_bpf_compute_max_units(50_000);

    program_test
}
