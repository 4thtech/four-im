use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{
            AccountMeta,
            Instruction,
        },
        pubkey::Pubkey,
        system_program,
        sysvar,
    },
    crate::{
        id,
        state::{
            User,
            Conversation,
            ConversationEncryptionInfo,
            Message,
            UserConversation,
        },
    },
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum InstantMessagingInstruction {
    /// Create a new user account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Funder account (must be a system account)
    /// 1. `[]` PDA address of the user
    /// 2. `[]` Wallet address of the user (must be a system account)
    /// 3. `[]` Rent sysvar
    /// 4. `[]` System program
    CreateUserAccount,

    /// Create a new conversation account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Funder account (must be a system account)
    /// 1. `[]` PDA address of the conversation
    /// 2. `[writable]` PDA address of the user - sender
    /// 3. `[writable]` PDA address of the user - receiver
    /// 4. `[writable]` PDA address of the user-conversation - sender
    /// 5. `[writable]` PDA address of the user-conversation - receiver
    /// 6. `[]` Rent sysvar
    /// 7. `[]` System program
    CreateConversationAccount,

    /// Create a new user-conversation account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Funder account (must be a system account)
    /// 1. `[]` PDA address of the user-conversation
    /// 2. `[]` PDA address of the user
    /// 3. `[]` Rent sysvar
    /// 4. `[]` System program
    CreateUserConversationAccount {
        conversation_index: u32,
    },

    /// Create a new message account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Funder account (must be a system account)
    /// 1. `[signer]` Sender account (must be a system account)
    /// 2. `[]` PDA address of the user - sender
    /// 3. `[]` PDA address of the user-conversation - sender
    /// 4. `[writable]` PDA address of the conversation
    /// 5. `[writable]` PDA address of the message
    /// 6. `[]` Rent sysvar
    /// 7. `[]` Clock sysvar
    /// 8. `[]` System program
    CreateMessageAccount {
        conversation_index: u32,
        message_type: u8,
        content: Vec<u8>,
    },

    /// Create a new conversation-encryption-info account account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Funder account (must be a system account)
    /// 1. `[signer]` Sender account (must be a system account)
    /// 2. `[writable]` PDA address of the conversation-encryption-info
    /// 3. `[]` PDA address of the user - sender
    /// 4. `[]` PDA address of the user - receiver
    /// 5. `[]` PDA address of the conversation
    /// 6. `[]` Rent sysvar
    /// 7. `[]` Clock sysvar
    /// 8. `[]` System program
    CreateConversationEncryptionInfoAccount {
        data: Vec<u8>,
    },

    /// Create a new message account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` Funder account (must be a system account)
    /// 1. `[signer]` Sender account (must be a system account)
    /// 2. `[writable]` PDA address of the user - sender
    /// 3. `[writable]` PDA address of the user - receiver
    /// 4. `[writable]` PDA address of the conversation
    /// 5. `[writable]` PDA address of the message
    /// 6. `[]` Rent sysvar
    /// 7. `[]` Clock sysvar
    /// 8. `[]` System program
    SendMessage {
        message_type: u8,
        content: Vec<u8>,
    },
}

/// Creates CreateUserAccount instruction
pub fn create_user_account(
    funder_address: &Pubkey,
    wallet_address: &Pubkey,
) -> Instruction {
    let user_pda_address = User::find_pda_address(
        &wallet_address,
        &id(),
    );

    Instruction::new_with_borsh(
        id(),
        &InstantMessagingInstruction::CreateUserAccount {},
        vec![
            AccountMeta::new(*funder_address, true),
            AccountMeta::new(user_pda_address, false),
            AccountMeta::new(*wallet_address, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

/// Creates CreateConversationAccount instruction
pub fn create_conversation_account(
    funder_address: &Pubkey,
    sender_pda_address: &Pubkey,
    receiver_pda_address: &Pubkey,
    sender_user_conversation_index: u32,
    receiver_user_conversation_index: u32,
) -> Instruction {
    let conversation_pda_address = Conversation::find_pda_address(
        &sender_pda_address,
        &receiver_pda_address,
        &id(),
    );

    let sender_user_conversation_pda_address = UserConversation::find_pda_address(
        &sender_pda_address,
        sender_user_conversation_index,
        &id(),
    );

    let receiver_user_conversation_pda_address = UserConversation::find_pda_address(
        &receiver_pda_address,
        receiver_user_conversation_index,
        &id(),
    );

    Instruction::new_with_borsh(
        id(),
        &InstantMessagingInstruction::CreateConversationAccount {},
        vec![
            AccountMeta::new(*funder_address, true),
            AccountMeta::new(conversation_pda_address, false),
            AccountMeta::new(*sender_pda_address, false),
            AccountMeta::new(*receiver_pda_address, false),
            AccountMeta::new(sender_user_conversation_pda_address, false),
            AccountMeta::new(receiver_user_conversation_pda_address, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

/// Creates CreateUserConversationAccount instruction
pub fn create_user_conversation_account(
    funder_address: &Pubkey,
    user_pda_address: &Pubkey,
    conversation_index: u32,
) -> Instruction {
    let user_conversation_pda_address = UserConversation::find_pda_address(
        &user_pda_address,
        conversation_index,
        &id(),
    );

    Instruction::new_with_borsh(
        id(),
        &InstantMessagingInstruction::CreateUserConversationAccount {
            conversation_index,
        },
        vec![
            AccountMeta::new(*funder_address, true),
            AccountMeta::new(user_conversation_pda_address, false),
            AccountMeta::new(*user_pda_address, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

/// Creates CreateMessageAccount instruction
pub fn create_message_account(
    funder_address: &Pubkey,
    sender_wallet_address: &Pubkey,
    conversation_pda_address: &Pubkey,
    conversation_index: u32,
    message_index: u32,
    message_type: u8,
    content: Vec<u8>,
) -> Instruction {
    let sender_user_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let sender_user_conversation_pda_address = UserConversation::find_pda_address(
        &sender_user_pda_address,
        conversation_index,
        &id(),
    );

    let message_pda_address = Message::find_pda_address(
        &conversation_pda_address,
        message_index,
        &id(),
    );

    Instruction::new_with_borsh(
        id(),
        &InstantMessagingInstruction::CreateMessageAccount {
            conversation_index,
            message_type,
            content,
        },
        vec![
            AccountMeta::new(*funder_address, true),
            AccountMeta::new(*sender_wallet_address, true),
            AccountMeta::new(sender_user_pda_address, false),
            AccountMeta::new(sender_user_conversation_pda_address, false),
            AccountMeta::new(*conversation_pda_address, false),
            AccountMeta::new(message_pda_address, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

/// Creates CreateConversationEncryptionInfoAccount instruction
pub fn create_conversation_encryption_info_account(
    funder_address: &Pubkey,
    sender_wallet_address: &Pubkey,
    receiver_wallet_address: &Pubkey,
    data: Vec<u8>,
) -> Instruction {
    let sender_user_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let receiver_user_pda_address = User::find_pda_address(
        &receiver_wallet_address,
        &id(),
    );

    let conversation_pda_address = Conversation::find_pda_address(
        &sender_user_pda_address,
        &receiver_user_pda_address,
        &id(),
    );

    let conversation_encryption_info_pda_address = ConversationEncryptionInfo::find_pda_address(
        &conversation_pda_address,
        &id(),
    );

    Instruction::new_with_borsh(
        id(),
        &InstantMessagingInstruction::CreateConversationEncryptionInfoAccount {
            data,
        },
        vec![
            AccountMeta::new(*funder_address, true),
            AccountMeta::new(*sender_wallet_address, true),
            AccountMeta::new(conversation_encryption_info_pda_address, false),
            AccountMeta::new(sender_user_pda_address, false),
            AccountMeta::new(receiver_user_pda_address, false),
            AccountMeta::new(conversation_pda_address, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}

/// Creates SendMessage instruction
pub fn send_message(
    funder_address: &Pubkey,
    sender_wallet_address: &Pubkey,
    receiver_wallet_address: &Pubkey,
    message_index: u32,
    message_type: u8,
    content: Vec<u8>,
) -> Instruction {
    let sender_user_pda_address = User::find_pda_address(
        &sender_wallet_address,
        &id(),
    );

    let receiver_user_pda_address = User::find_pda_address(
        &receiver_wallet_address,
        &id(),
    );

    let conversation_pda_address = Conversation::find_pda_address(
        &sender_user_pda_address,
        &receiver_user_pda_address,
        &id(),
    );

    let message_pda_address = Message::find_pda_address(
        &conversation_pda_address,
        message_index,
        &id(),
    );

    Instruction::new_with_borsh(
        id(),
        &InstantMessagingInstruction::SendMessage {
            message_type,
            content,
        },
        vec![
            AccountMeta::new(*funder_address, true),
            AccountMeta::new(*sender_wallet_address, true),
            AccountMeta::new(sender_user_pda_address, false),
            AccountMeta::new(receiver_user_pda_address, false),
            AccountMeta::new(conversation_pda_address, false),
            AccountMeta::new(message_pda_address, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(sysvar::clock::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    )
}
