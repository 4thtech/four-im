// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
// #![cfg(feature = "test-bpf")]

mod program_test;
mod utils;

use solana_sdk::signer::keypair::Keypair;
use {
    utils::{
        create_user_pda_account,
        create_conversation_pda_account,
        create_user_conversation_pda_account,
        create_conversation_encryption_info_pda_account,
        send_message,
    },
    borsh::{BorshDeserialize},
    solana_program::{
        pubkey::Pubkey,
        sysvar,
    },
    solana_program_test::*,
    solana_sdk::{
        signature::{
            Signer,
        },
    },
    program_test::program_test,
    instant_messaging::{
        id,
        state::{
            User,
            Conversation,
            ConversationEncryptionInfo,
            Message,
            MessageType,
        },
    },
};
use instant_messaging::state::UserConversation;
use crate::utils::create_message_pda_account;

//#[tokio::test]
async fn test_create_user_account() {
    let user_wallet_address = Pubkey::new_unique();
    let user_pda_address = User::find_pda_address(
        &user_wallet_address,
        &id(),
    );

    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    // User PDA account does not exist
    assert_eq!(
        banks_client
            .get_account(user_pda_address)
            .await
            .expect("get_account"),
        None,
    );

    // Create User PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &user_wallet_address,
    ).await;

    // User PDA account now exist
    let user_pda_account = banks_client
        .get_account(user_pda_address)
        .await
        .expect("get_account")
        .expect("user_pda_account not found");

    assert_eq!(
        user_pda_account.data.len(),
        User::retrieve_size(),
    );

    assert_eq!(
        User::try_from_slice(&user_pda_account.data)
            .unwrap()
            .conversation_counter,
        0,
    );

    let is_rent_exempt = sysvar::rent::Rent::default()
        .is_exempt(user_pda_account.lamports, user_pda_account.data.len());

    assert_eq!(is_rent_exempt, true);
}

//#[tokio::test]
async fn test_conversation_pda_address_generation() {
    let sender_pda_address = Pubkey::new_unique();
    let receiver_pda_address = Pubkey::new_unique();

    let conversation_pda_address_1 = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    let conversation_pda_address_2 = Conversation::find_pda_address(
        &receiver_pda_address,
        &sender_pda_address,
        &id(),
    );

    assert_eq!(conversation_pda_address_1, conversation_pda_address_2);
}

//#[tokio::test]
async fn test_create_conversation_account() {
    let sender_pda_address = Pubkey::new_unique();
    let receiver_pda_address = Pubkey::new_unique();

    let conversation_pda_address = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    // Create User PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_pda_address,
    ).await;

    // Create User PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &receiver_pda_address,
    ).await;

    // Conversation PDA account does not exist
    assert_eq!(
        banks_client
            .get_account(conversation_pda_address)
            .await
            .expect("get_account"),
        None,
    );

    let sender_user_conversation_index = 0;
    let receiver_user_conversation_index = 0;

    // Create Conversation PDA account
    create_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_pda_address,
        &receiver_pda_address,
        sender_user_conversation_index,
        receiver_user_conversation_index,
    ).await;

    // Conversation PDA account now exist
    let conversation_pda_account = banks_client
        .get_account(conversation_pda_address)
        .await
        .expect("get_account")
        .expect("conversation_pda_account not found");

    assert_eq!(
        conversation_pda_account.data.len(),
        Conversation::retrieve_size(),
    );

    assert_eq!(
        Conversation::try_from_slice(&conversation_pda_account.data)
            .unwrap()
            .message_counter,
        0,
    );

    let is_rent_exempt = sysvar::rent::Rent::default()
        .is_exempt(conversation_pda_account.lamports, conversation_pda_account.data.len());

    assert_eq!(is_rent_exempt, true);

    // Sender's user PDA account should have 1 conversation
    let sender_user_pda_account = banks_client
        .get_account(sender_pda_address)
        .await
        .expect("get_account")
        .expect("sender_user_pda_account not found");

    assert_eq!(
        User::try_from_slice(&sender_user_pda_account.data)
            .unwrap()
            .conversation_counter,
        1,
    );

    // Receiver's user PDA account should have 1 conversation
    let receiver_user_pda_account = banks_client
        .get_account(receiver_pda_address)
        .await
        .expect("get_account")
        .expect("receiver_user_pda_account not found");

    assert_eq!(
        User::try_from_slice(&receiver_user_pda_account.data)
            .unwrap()
            .conversation_counter,
        1,
    );

    // Sender's user-conversation PDA account should have conversation address
    let sender_user_conversation_pda_address = UserConversation::find_pda_address(
        &sender_pda_address,
        sender_user_conversation_index,
        &id(),
    );

    let sender_user_conversation_pda_account = banks_client
        .get_account(sender_user_conversation_pda_address)
        .await
        .expect("get_account")
        .expect("sender_user_conversation_pda_account not found");

    assert_eq!(
        UserConversation::try_from_slice(&sender_user_conversation_pda_account.data)
            .unwrap()
            .conversation_address,
        conversation_pda_address,
    );

    // Receiver's user-conversation PDA account should have conversation address
    let receiver_user_conversation_pda_address = UserConversation::find_pda_address(
        &receiver_pda_address,
        receiver_user_conversation_index,
        &id(),
    );

    let receiver_user_conversation_pda_account = banks_client
        .get_account(receiver_user_conversation_pda_address)
        .await
        .expect("get_account")
        .expect("sender_user_conversation_pda_account not found");

    assert_eq!(
        UserConversation::try_from_slice(&receiver_user_conversation_pda_account.data)
            .unwrap()
            .conversation_address,
        conversation_pda_address,
    );
}

//#[tokio::test]
async fn test_create_user_conversation_account() {
    let user_pda_address = Pubkey::new_unique();
    let conversation_index = 0;

    let user_conversation_pda_address = UserConversation::find_pda_address(
        &user_pda_address,
        conversation_index,
        &id(),
    );

    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    // UserConversation PDA account does not exist
    assert_eq!(
        banks_client
            .get_account(user_conversation_pda_address)
            .await
            .expect("get_account"),
        None,
    );

    create_user_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &user_pda_address,
        conversation_index,
    ).await;

    // UserConversation PDA account now exist
    let user_conversation_pda_account = banks_client
        .get_account(user_conversation_pda_address)
        .await
        .expect("get_account")
        .expect("user_conversation_pda_account not found");

    assert_eq!(
        user_conversation_pda_account.data.len(),
        UserConversation::retrieve_size(),
    );

    let is_rent_exempt = sysvar::rent::Rent::default()
        .is_exempt(user_conversation_pda_account.lamports, user_conversation_pda_account.data.len());

    assert_eq!(is_rent_exempt, true);
}

// #[tokio::test]
async fn create_message_account() {
    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    let sender = Keypair::new();
    let sender_wallet_address = sender.pubkey();
    let receiver_wallet_address = Pubkey::new_unique();

    let sender_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let receiver_pda_address = User::find_pda_address(
        &receiver_wallet_address,
        &id(),
    );

    // Create sender's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_wallet_address,
    ).await;

    // Create receiver's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &receiver_wallet_address,
    ).await;

    // Create Conversation PDA account
    let sender_user_conversation_index = 0;
    let receiver_user_conversation_index = 0;

    create_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_pda_address,
        &receiver_pda_address,
        sender_user_conversation_index,
        receiver_user_conversation_index,
    ).await;

    let conversation_pda_address = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    let message_index = 0;
    let message_type = MessageType::PLAIN_TEXT;
    let message_content = String::from("First message!").into_bytes();

    create_message_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender,
        &conversation_pda_address,
        sender_user_conversation_index,
        message_index,
        message_type,
        &message_content,
    ).await;

    let message_pda_address = Message::find_pda_address(
        &conversation_pda_address,
        message_index,
        &id(),
    );

    let message_pda_account = banks_client
        .get_account(message_pda_address)
        .await
        .expect("get_account")
        .expect("message_pda_account not found");

    let message: Message = Message::try_from_slice(&message_pda_account.data)
        .unwrap();

    assert_eq!(message.sender, sender.pubkey());
    assert_eq!(message.message_type, message_type);
    assert_eq!(message.content, message_content);
    assert_ne!(message.timestamp, 0);

    // Conversation message counter should be 1
    let conversation_pda_account = banks_client
        .get_account(conversation_pda_address)
        .await
        .expect("get_account")
        .expect("conversation_pda_account not found");

    assert_eq!(
        Conversation::try_from_slice(&conversation_pda_account.data)
            .unwrap()
            .message_counter,
        1,
    );
}

//#[tokio::test]
#[should_panic]
async fn test_create_message_to_wrong_conversation() {
    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    let sender = Keypair::new();
    let sender_wallet_address = sender.pubkey();
    let receiver_wallet_address = Pubkey::new_unique();

    let sender_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let receiver_pda_address = User::find_pda_address(
        &receiver_wallet_address,
        &id(),
    );

    // Create sender's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_wallet_address,
    ).await;

    // Create receiver's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &receiver_wallet_address,
    ).await;

    // Create Conversation PDA account
    let sender_user_conversation_index = 0;
    let receiver_user_conversation_index = 0;

    create_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_pda_address,
        &receiver_pda_address,
        sender_user_conversation_index,
        receiver_user_conversation_index,
    ).await;

    let correct_conversation_pda_address = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    // Create wrong conversation
    let wrong_sender = Pubkey::new_unique();
    let wrong_receiver = Pubkey::new_unique();

    let wrong_sender_pda_address = User::find_pda_address(
        &wrong_sender,
        &id(),
    );

    let wrong_receiver_pda_address = User::find_pda_address(
        &wrong_receiver,
        &id(),
    );

    // Create sender's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &wrong_sender,
    ).await;

    // Create receiver's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &wrong_receiver,
    ).await;

    let wrong_conversation_pda_address = Conversation::find_pda_address(
        &wrong_sender_pda_address,
        &wrong_receiver_pda_address,
        &id(),
    );
    create_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &wrong_sender_pda_address,
        &wrong_receiver_pda_address,
        sender_user_conversation_index,
        receiver_user_conversation_index,
    ).await;

    assert_ne!(correct_conversation_pda_address, wrong_conversation_pda_address);

    let message_index = 0;
    let message_type = MessageType::PLAIN_TEXT;
    let message_content = String::from("First message!").into_bytes();

    create_message_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender,
        &wrong_conversation_pda_address,
        sender_user_conversation_index,
        message_index,
        message_type,
        &message_content,
    ).await;
}

//#[tokio::test]
async fn test_create_conversation_encryption_info_account() {
    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    let sender = Keypair::new();
    let sender_wallet_address = sender.pubkey();
    let receiver_wallet_address = Pubkey::new_unique();

    let sender_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let receiver_pda_address = User::find_pda_address(
        &receiver_wallet_address,
        &id(),
    );

    let conversation_pda_address = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    let encryption_pda_address = ConversationEncryptionInfo::find_pda_address(
        &conversation_pda_address,
        &id(),
    );

    // Create sender's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_wallet_address,
    ).await;

    // Create receiver's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &receiver_wallet_address,
    ).await;

    // Create Conversation PDA account
    let sender_user_conversation_index = 0;
    let receiver_user_conversation_index = 0;

    create_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_pda_address,
        &receiver_pda_address,
        sender_user_conversation_index,
        receiver_user_conversation_index,
    ).await;

    // Create account
    let data = String::from("{ type: rsa, addresses: [82FAYgrex2cq2SZkJzUZSfVFf4j3jxHLZonbX63EXPey,82FAYgrex2cq2SZkJzUZSfVFf4j3jxHLZonbX63EXPey]}").into_bytes();
    create_conversation_encryption_info_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender,
        &receiver_wallet_address,
        &data,
    ).await;

    // Message PDA account now exist
    let encryption_info_pda_account = banks_client
        .get_account(encryption_pda_address)
        .await
        .expect("get_account")
        .expect("encryption_info_pda_account not found");

    let encryption_info: ConversationEncryptionInfo = ConversationEncryptionInfo::try_from_slice(&encryption_info_pda_account.data)
        .unwrap();

    assert_eq!(encryption_info.data, data);
}

#[tokio::test]
async fn test_send_message() {
    let sender = Keypair::new();
    let sender_wallet_address = sender.pubkey();
    let receiver_wallet_address = Pubkey::new_unique();

    let sender_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let receiver_pda_address = User::find_pda_address(
        &receiver_wallet_address,
        &id(),
    );

    let conversation_pda_address = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    let (mut banks_client, payer, recent_blockhash) =
        program_test().start().await;

    // Create sender's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_wallet_address,
    ).await;

    // Create receiver's user PDA account
    create_user_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &receiver_wallet_address,
    ).await;

    // Create Conversation PDA account
    let sender_user_conversation_index = 0;
    let receiver_user_conversation_index = 0;

    create_conversation_pda_account(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender_pda_address,
        &receiver_pda_address,
        sender_user_conversation_index,
        receiver_user_conversation_index,
    ).await;

    // Sender's user PDA account should have 1 conversation
    let sender_user_pda_account = banks_client
        .get_account(sender_pda_address)
        .await
        .expect("get_account")
        .expect("sender_user_pda_account not found");

    assert_eq!(
        User::try_from_slice(&sender_user_pda_account.data)
            .unwrap()
            .conversation_counter,
        1,
    );

    // Receiver's user PDA account should have 1 conversation
    let receiver_user_pda_account = banks_client
        .get_account(receiver_pda_address)
        .await
        .expect("get_account")
        .expect("receiver_user_pda_account not found");

    assert_eq!(
        User::try_from_slice(&receiver_user_pda_account.data)
            .unwrap()
            .conversation_counter,
        1,
    );

    // Sender's user-conversation PDA account should have conversation address
    let sender_user_conversation_pda_address = UserConversation::find_pda_address(
        &sender_pda_address,
        sender_user_conversation_index,
        &id(),
    );

    let sender_user_conversation_pda_account = banks_client
        .get_account(sender_user_conversation_pda_address)
        .await
        .expect("get_account")
        .expect("sender_user_conversation_pda_account not found");

    assert_eq!(
        UserConversation::try_from_slice(&sender_user_conversation_pda_account.data)
            .unwrap()
            .conversation_address,
        conversation_pda_address,
    );

    // Receiver's user-conversation PDA account should have conversation address
    let receiver_user_conversation_pda_address = UserConversation::find_pda_address(
        &receiver_pda_address,
        receiver_user_conversation_index,
        &id(),
    );

    let receiver_user_conversation_pda_account = banks_client
        .get_account(receiver_user_conversation_pda_address)
        .await
        .expect("get_account")
        .expect("sender_user_conversation_pda_account not found");

    assert_eq!(
        UserConversation::try_from_slice(&receiver_user_conversation_pda_account.data)
            .unwrap()
            .conversation_address,
        conversation_pda_address,
    );

    // Send 1. message
    let first_message_index = 0;
    let first_message_type = MessageType::PLAIN_TEXT;
    let first_message_content = String::from("First message!").into_bytes();

    let message_pda_address = Message::find_pda_address(
        &conversation_pda_address,
        first_message_index,
        &id(),
    );

    // Message PDA account does not exist
    assert_eq!(
        banks_client
            .get_account(message_pda_address)
            .await
            .expect("get_account"),
        None,
    );

    send_message(
        &payer,
        &mut banks_client,
        &recent_blockhash,
        &sender,
        &receiver_wallet_address,
        first_message_index,
        first_message_type,
        &first_message_content,
    ).await;

    // Message PDA account now exist
    let message_pda_account = banks_client
        .get_account(message_pda_address)
        .await
        .expect("get_account")
        .expect("message_pda_account not found");

    let first_message: Message = Message::try_from_slice(&message_pda_account.data)
        .unwrap();

    assert_eq!(first_message.sender, sender.pubkey());
    assert_eq!(first_message.message_type, first_message_type);
    assert_eq!(first_message.content, first_message_content);
    assert_ne!(first_message.timestamp, 0);

    // Conversation message counter should be 1
    let conversation_pda_account = banks_client
        .get_account(conversation_pda_address)
        .await
        .expect("get_account")
        .expect("conversation_pda_account not found");

    assert_eq!(
        Conversation::try_from_slice(&conversation_pda_account.data)
            .unwrap()
            .message_counter,
        1,
    );
}
