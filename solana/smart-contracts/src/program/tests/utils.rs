use {
    solana_program::{
        hash::Hash,
        pubkey::Pubkey,
    },
    solana_program_test::BanksClient,
    solana_sdk::{
        signature::{
            Keypair,
            Signer,
        },
        transaction::Transaction,
    },
    instant_messaging::instruction::{
        create_user_account,
        create_conversation_account,
        create_user_conversation_account,
        create_message_account,
        send_message as send_message_instruction,
    },
};
use instant_messaging::instruction::create_conversation_encryption_info_account;

/// Creates User PDA account
pub async fn create_user_pda_account(
    payer: &Keypair,
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    user_wallet_address: &Pubkey,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[create_user_account(
            &payer.pubkey(),
            &user_wallet_address,
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );

    banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

/// Creates Conversation PDA account
pub async fn create_conversation_pda_account(
    payer: &Keypair,
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    sender_pda_address: &Pubkey,
    receiver_pda_address: &Pubkey,
    sender_user_conversation_index: u32,
    receiver_user_conversation_index: u32,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[create_conversation_account(
            &payer.pubkey(),
            &sender_pda_address,
            &receiver_pda_address,
            sender_user_conversation_index,
            receiver_user_conversation_index,
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );

    banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

/// Creates UserConversation PDA account
pub async fn create_user_conversation_pda_account(
    payer: &Keypair,
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    user_pda_address: &Pubkey,
    conversation_index: u32,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[create_user_conversation_account(
            &payer.pubkey(),
            &user_pda_address,
            conversation_index,
        )],
        Some(&payer.pubkey()),
        &[payer],
        *recent_blockhash,
    );

    banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

/// Creates Message PDA account
pub async fn create_message_pda_account(
    payer: &Keypair,
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    sender: &Keypair,
    conversation_pda_address: &Pubkey,
    conversation_index: u32,
    message_index: u32,
    message_type: u8,
    content: &Vec<u8>,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[create_message_account(
            &payer.pubkey(),
            &sender.pubkey(),
            &conversation_pda_address,
            conversation_index,
            message_index,
            message_type,
            content.clone(),
        )],
        Some(&payer.pubkey()),
        &[payer, sender],
        *recent_blockhash,
    );

    banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

/// Creates ConversationEncryptionInfo PDA account
pub async fn create_conversation_encryption_info_pda_account(
    payer: &Keypair,
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    sender: &Keypair,
    receiver_wallet_address: &Pubkey,
    data: &Vec<u8>,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[create_conversation_encryption_info_account(
            &payer.pubkey(),
            &sender.pubkey(),
            receiver_wallet_address,
            data.clone(),
        )],
        Some(&payer.pubkey()),
        &[payer, sender],
        *recent_blockhash,
    );

    banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

/// Send Message
pub async fn send_message(
    payer: &Keypair,
    banks_client: &mut BanksClient,
    recent_blockhash: &Hash,
    sender: &Keypair,
    receiver_wallet_address: &Pubkey,
    message_index: u32,
    message_type: u8,
    content: &Vec<u8>,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[send_message_instruction(
            &payer.pubkey(),
            &sender.pubkey(),
            receiver_wallet_address,
            message_index,
            message_type,
            content.clone(),
        )],
        Some(&payer.pubkey()),
        &[payer, sender],
        *recent_blockhash,
    );

    banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}