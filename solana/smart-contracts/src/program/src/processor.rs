//! Program state processor

use {
    crate::{
        instruction::InstantMessagingInstruction,
        state::{
            User,
            Conversation,
            UserConversation,
            sort_addresses_asc,
        },
        utils::create_pda_account,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{
            next_account_info,
            AccountInfo,
        },
        clock::Clock,
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::{
            clock,
            rent,
            Sysvar,
        },
    },
};
use crate::state::{ConversationEncryptionInfo, Message};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = InstantMessagingInstruction::try_from_slice(instruction_data)?;

    match instruction {
        InstantMessagingInstruction::CreateUserAccount {} => create_user_account(program_id, accounts),
        InstantMessagingInstruction::CreateConversationAccount {} => create_conversation_account(program_id, accounts),
        InstantMessagingInstruction::CreateUserConversationAccount {
            conversation_index,
        } => create_user_conversation_account(program_id, accounts, conversation_index),
        InstantMessagingInstruction::CreateMessageAccount {
            conversation_index,
            message_type,
            content,
        } => create_message_account(
            program_id,
            accounts,
            conversation_index,
            message_type,
            content,
        ),
        InstantMessagingInstruction::CreateConversationEncryptionInfoAccount {
            data,
        } => create_conversation_encryption_info_account(
            program_id,
            accounts,
            data,
        ),
        InstantMessagingInstruction::SendMessage {
            message_type,
            content,
        } => send_message(
            program_id,
            accounts,
            message_type,
            content,
        ),
    }
}

fn create_user_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let user_account_info = next_account_info(account_info_iter)?;
    let user_wallet_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_info)?;

    let (user_address, user_bump_seed) =
        User::find_pda_address_with_bump_seed(
            user_wallet_account_info.key,
            program_id,
        );

    if user_address != *user_account_info.key {
        msg!("Error: User address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    if user_account_info.data.borrow().len() > 0 {
        msg!("Error: User account is already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let user_account_signer_seeds: &[&[_]] = &[
        &user_wallet_account_info.key.to_bytes(),
        &User::ACCOUNT_ADDRESS_SEED.as_bytes(),
        &[user_bump_seed],
    ];

    create_pda_account(
        funder_info,
        &rent,
        true,
        User::retrieve_size(),
        program_id,
        system_program_info,
        user_account_info,
        user_account_signer_seeds,
    )?;

    Ok(())
}

fn create_conversation_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let conversation_account_info = next_account_info(account_info_iter)?;
    let first_user_pda_account_info = next_account_info(account_info_iter)?;
    let second_user_pda_account_info = next_account_info(account_info_iter)?;
    let sender_user_conversation_account_info = next_account_info(account_info_iter)?;
    let receiver_user_conversation_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_info)?;

    let (conversation_address, conversation_bump_seed) =
        Conversation::find_pda_address_with_bump_seed(
            first_user_pda_account_info.key,
            second_user_pda_account_info.key,
            program_id,
        );

    if conversation_address != *conversation_account_info.key {
        msg!("Error: Conversation address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    if conversation_account_info.data.borrow().len() > 0 {
        msg!("Error: Conversation account is already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Sort first & second address so that they are always in the same order
    let [address_one, address_two] = sort_addresses_asc(
        first_user_pda_account_info.key,
        second_user_pda_account_info.key,
    );

    let conversation_account_signer_seeds: &[&[_]] = &[
        &address_one.to_bytes(),
        &address_two.to_bytes(),
        &Conversation::ACCOUNT_ADDRESS_SEED.as_bytes(),
        &[conversation_bump_seed],
    ];

    create_pda_account(
        funder_info,
        &rent,
        true,
        Conversation::retrieve_size(),
        program_id,
        system_program_info,
        conversation_account_info,
        conversation_account_signer_seeds,
    )?;

    // TODO: Create sender's user-conversation account if not exist
    let mut sender_user = User::try_from_slice(&first_user_pda_account_info.data.borrow())?;

    if sender_user_conversation_account_info.data.borrow().len() == 0 {
        create_user_conversation_account(
            program_id,
            &[
                funder_info.clone(),
                sender_user_conversation_account_info.clone(),
                first_user_pda_account_info.clone(),
                rent_info.clone(),
                system_program_info.clone(),
            ],
            sender_user.conversation_counter,
        )?;

        // Assign conversation address to sender user-conversation account
        let mut sender_user_conversation = UserConversation::try_from_slice(&sender_user_conversation_account_info.data.borrow())?;
        sender_user_conversation.conversation_address = *conversation_account_info.key;
        sender_user_conversation.serialize(&mut &mut sender_user_conversation_account_info.data.borrow_mut()[..])?;

        // Increment and store the number of conversations the sender user account has
        sender_user.conversation_counter += 1;
        sender_user.serialize(&mut &mut first_user_pda_account_info.data.borrow_mut()[..])?;
    }

    // TODO: Create receiver's user-conversation account if not exist
    let mut receiver_user = User::try_from_slice(&second_user_pda_account_info.data.borrow())?;

    if receiver_user_conversation_account_info.data.borrow().len() == 0 {
        create_user_conversation_account(
            program_id,
            &[
                funder_info.clone(),
                receiver_user_conversation_account_info.clone(),
                second_user_pda_account_info.clone(),
                rent_info.clone(),
                system_program_info.clone(),
            ],
            receiver_user.conversation_counter,
        )?;

        // Assign conversation address to receiver user-conversation account
        let mut receiver_user_conversation = UserConversation::try_from_slice(&receiver_user_conversation_account_info.data.borrow())?;
        receiver_user_conversation.conversation_address = *conversation_account_info.key;
        receiver_user_conversation.serialize(&mut &mut receiver_user_conversation_account_info.data.borrow_mut()[..])?;

        // Increment and store the number of conversations the receiver user account has
        receiver_user.conversation_counter += 1;
        receiver_user.serialize(&mut &mut second_user_pda_account_info.data.borrow_mut()[..])?;
    }

    Ok(())
}

fn create_user_conversation_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    conversation_index: u32,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let user_conversation_account_info = next_account_info(account_info_iter)?;
    let user_pda_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_info)?;

    let (user_conversation_address, user_conversation_bump_seed) =
        UserConversation::find_pda_address_with_bump_seed(
            user_pda_account_info.key,
            conversation_index,
            program_id,
        );

    if user_conversation_address != *user_conversation_account_info.key {
        msg!("Error: UserConversation address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    if user_conversation_account_info.data.borrow().len() > 0 {
        msg!("Error: UserConversation account is already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let account_seed = conversation_index.to_string() + UserConversation::ACCOUNT_ADDRESS_SEED;

    let user_conversation_account_signer_seeds: &[&[_]] = &[
        &user_pda_account_info.key.to_bytes(),
        &account_seed.as_bytes(),
        &[user_conversation_bump_seed],
    ];

    create_pda_account(
        funder_info,
        &rent,
        true,
        UserConversation::retrieve_size(),
        program_id,
        system_program_info,
        user_conversation_account_info,
        user_conversation_account_signer_seeds,
    )?;

    Ok(())
}

fn create_message_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    conversation_index: u32,
    message_type: u8,
    content: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let sender_info = next_account_info(account_info_iter)?;
    let sender_user_account_info = next_account_info(account_info_iter)?;
    let sender_user_conversation_account_info = next_account_info(account_info_iter)?;
    let conversation_account_info = next_account_info(account_info_iter)?;
    let message_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_info)?;

    // Check sender signature
    if !sender_info.is_signer {
        msg!("Error: Sender signature missing");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check if sender's user pda account exists
    if sender_user_account_info.data.borrow().len() == 0 {
        msg!("Error: Sender's user account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check if conversation account exists
    if conversation_account_info.data.borrow().len() == 0 {
        msg!("Error: Conversation account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check is conversation account has correct program ids
    if conversation_account_info.owner != program_id {
        msg!("Error: Conversation account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check seed derivations for user conversation account
    if *sender_user_conversation_account_info.key != UserConversation::find_pda_address(
        sender_user_account_info.key,
        conversation_index,
        program_id,
    ) {
        msg!("Error: UserConversation address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    // Check if user has access to conversation
    // -> Find UserConversation PDA address
    let user_conversation = UserConversation::try_from_slice(&sender_user_conversation_account_info.data.borrow())?;
    if *conversation_account_info.key != user_conversation.conversation_address {
        msg!("Error: Sender is not connected with this conversation.");
        return Err(ProgramError::IllegalOwner);
    }


    // Check rent system account
    if !rent::check_id(rent_info.key) {
        msg!("Error: Invalid rent system account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Check clock system account
    if !clock::check_id(clock_info.key) {
        msg!("Error: Invalid clock system account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Get conversation message counter
    let mut conversation: Conversation = Conversation::try_from_slice(&conversation_account_info.data.borrow())?;
    let message_counter = conversation.message_counter;

    // Check seed derivations for message account
    let (message_pda_address, message_bump_seed) = Message::find_pda_address_with_bump_seed(
        &conversation_account_info.key,
        message_counter,
        program_id,
    );

    if message_pda_address != *message_account_info.key {
        msg!("Error: Message address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    // Create message PDA account
    let account_seed = message_counter.to_string() + Message::ACCOUNT_ADDRESS_SEED;

    let message_account_signer_seeds: &[&[_]] = &[
        &conversation_account_info.key.to_bytes(),
        &account_seed.as_bytes(),
        &[message_bump_seed],
    ];

    create_pda_account(
        funder_info,
        &rent,
        false,
        Message::retrieve_size(content.len()),
        program_id,
        system_program_info,
        message_account_info,
        message_account_signer_seeds,
    )?;

    // Assign data to message
    let mut message = Message::new(content.len());
    message.sender = *sender_info.key;
    message.message_type = message_type;
    message.content = content;
    message.timestamp = Clock::from_account_info(clock_info)?.unix_timestamp;
    message.serialize(&mut &mut message_account_info.data.borrow_mut()[..])?;

    // Increment and store the number of messages the conversation account has
    conversation.message_counter += 1;
    conversation.serialize(&mut &mut conversation_account_info.data.borrow_mut()[..])?;

    Ok(())
}

fn create_conversation_encryption_info_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let sender_info = next_account_info(account_info_iter)?;
    let conversation_encryption_info_account_info = next_account_info(account_info_iter)?;
    let sender_user_account_info = next_account_info(account_info_iter)?;
    let receiver_user_account_info = next_account_info(account_info_iter)?;
    let conversation_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_info)?;

    let (conversation_encryption_info_address, conversation_encryption_info_bump_seed) =
        ConversationEncryptionInfo::find_pda_address_with_bump_seed(
            conversation_account_info.key,
            program_id,
        );

    if conversation_encryption_info_address != *conversation_encryption_info_account_info.key {
        msg!("Error: ConversationEncryptionInfo address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    // Check sender signature
    if !sender_info.is_signer {
        msg!("Error: Sender signature missing");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check if conversation account exists
    if conversation_account_info.data.borrow().len() == 0 {
        msg!("Error: Conversation account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check is sender's user account has correct program ids
    if sender_user_account_info.owner != program_id {
        msg!("Error: Sender's user account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check is receiver's user account has correct program ids
    if receiver_user_account_info.owner != program_id {
        msg!("Error: Receiver's user account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check is conversation account has correct program ids
    if conversation_account_info.owner != program_id {
        msg!("Error: Conversation account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check seed derivations for conversation account
    if *conversation_account_info.key != Conversation::find_pda_address(
        sender_user_account_info.key,
        receiver_user_account_info.key,
        program_id,
    ) {
        msg!("Error: Conversation address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    // Check rent system account
    if !rent::check_id(rent_info.key) {
        msg!("Error: Invalid rent system account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Check clock system account
    if !clock::check_id(clock_info.key) {
        msg!("Error: Invalid clock system account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Create conversation-encryption-info PDA account
    let encryption_account_signer_seeds: &[&[_]] = &[
        &conversation_account_info.key.to_bytes(),
        &ConversationEncryptionInfo::ACCOUNT_ADDRESS_SEED.as_bytes(),
        &[conversation_encryption_info_bump_seed],
    ];

    create_pda_account(
        funder_info,
        &rent,
        true,
        ConversationEncryptionInfo::retrieve_size(data.len()),
        program_id,
        system_program_info,
        conversation_encryption_info_account_info,
        encryption_account_signer_seeds,
    )?;

    // Assign data to message
    let mut account = ConversationEncryptionInfo::new(data.len());
    account.data = data;
    account.serialize(&mut &mut conversation_encryption_info_account_info.data.borrow_mut()[..])?;

    Ok(())
}

fn send_message(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    message_type: u8,
    content: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let funder_info = next_account_info(account_info_iter)?;
    let sender_info = next_account_info(account_info_iter)?;
    let sender_user_account_info = next_account_info(account_info_iter)?;
    let receiver_user_account_info = next_account_info(account_info_iter)?;
    let conversation_account_info = next_account_info(account_info_iter)?;
    let message_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_info)?;

    // Check sender signature
    if !sender_info.is_signer {
        msg!("Error: Sender signature missing");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check if sender's user pda account exists
    if sender_user_account_info.data.borrow().len() == 0 {
        msg!("Error: Sender's user account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check if receiver's user pda account exists
    if receiver_user_account_info.data.borrow().len() == 0 {
        msg!("Error: Receiver's user account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check if conversation account exists
    if conversation_account_info.data.borrow().len() == 0 {
        msg!("Error: Conversation account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check is sender's user account has correct program ids
    if sender_user_account_info.owner != program_id {
        msg!("Error: Sender's user account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check is receiver's user account has correct program ids
    if receiver_user_account_info.owner != program_id {
        msg!("Error: Receiver's user account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check is conversation account has correct program ids
    if conversation_account_info.owner != program_id {
        msg!("Error: Conversation account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check seed derivations for conversation account
    if *conversation_account_info.key != Conversation::find_pda_address(
        sender_user_account_info.key,
        receiver_user_account_info.key,
        program_id,
    ) {
        msg!("Error: Conversation address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    // Check rent system account
    if !rent::check_id(rent_info.key) {
        msg!("Error: Invalid rent system account");
        return Err(ProgramError::InvalidAccountData);
    }

    // Check clock system account
    if !clock::check_id(clock_info.key) {
        msg!("Error: Invalid clock system account");
        return Err(ProgramError::InvalidAccountData);
    }

    // TODO: ?

    // Get conversation message counter
    let mut conversation: Conversation = Conversation::try_from_slice(&conversation_account_info.data.borrow())?;
    let message_counter = conversation.message_counter;

    // Check seed derivations for message account
    let (message_pda_address, message_bump_seed) = Message::find_pda_address_with_bump_seed(
        &conversation_account_info.key,
        message_counter,
        program_id,
    );

    if message_pda_address != *message_account_info.key {
        msg!("Error: Message address does not match seed derivation");
        return Err(ProgramError::InvalidSeeds);
    }

    // Create message PDA account
    let account_seed = message_counter.to_string() + Message::ACCOUNT_ADDRESS_SEED;

    let message_account_signer_seeds: &[&[_]] = &[
        &conversation_account_info.key.to_bytes(),
        &account_seed.as_bytes(),
        &[message_bump_seed],
    ];

    create_pda_account(
        funder_info,
        &rent,
        false,
        Message::retrieve_size(content.len()),
        program_id,
        system_program_info,
        message_account_info,
        message_account_signer_seeds,
    )?;

    // Assign data to message
    let mut message = Message::new(content.len());
    message.sender = *sender_info.key;
    message.message_type = message_type;
    message.content = content;
    message.timestamp = Clock::from_account_info(clock_info)?.unix_timestamp;
    message.serialize(&mut &mut message_account_info.data.borrow_mut()[..])?;

    // Increment and store the number of messages the conversation account has
    conversation.message_counter += 1;
    conversation.serialize(&mut &mut conversation_account_info.data.borrow_mut()[..])?;

    Ok(())
}
