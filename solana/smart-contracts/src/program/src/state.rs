use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        clock::UnixTimestamp,
        pubkey::{
            Pubkey,
        },
    },
    std::mem,
};

/// Define a user account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct User {
    /// Number of conversations the user has
    pub conversation_counter: u32,
}

impl User {
    pub const ACCOUNT_ADDRESS_SEED: &'static str = "user";

    /// Get size of user account
    pub fn retrieve_size() -> usize {
        mem::size_of::<User>()
    }

    /// Get program-derived account address and bump seeds for the user
    pub fn find_pda_address_with_bump_seed(
        user_wallet_address: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &user_wallet_address.to_bytes(),
                Self::ACCOUNT_ADDRESS_SEED.as_bytes(),
            ],
            program_id,
        )
    }

    /// Get program-derived account address for the user
    pub fn find_pda_address(
        user_wallet_address: &Pubkey,
        program_id: &Pubkey,
    ) -> Pubkey {
        Self::find_pda_address_with_bump_seed(user_wallet_address, program_id).0
    }
}

/// Define a conversation account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Conversation {
    /// Number of messages that conversation has
    pub message_counter: u32,
}

impl Conversation {
    pub const ACCOUNT_ADDRESS_SEED: &'static str = "conversation";

    /// Get size of conversation account
    pub fn retrieve_size() -> usize {
        mem::size_of::<Conversation>()
    }

    /// Get program-derived account address and bump seeds for the conversation
    pub fn find_pda_address_with_bump_seed(
        first_user_pda_address: &Pubkey,
        second_user_pda_address: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        // Sort first & second address so that they are always in the same order
        let [address_one, address_two] = sort_addresses_asc(
            first_user_pda_address,
            second_user_pda_address,
        );

        Pubkey::find_program_address(
            &[
                &address_one.to_bytes(),
                &address_two.to_bytes(),
                Self::ACCOUNT_ADDRESS_SEED.as_bytes(),
            ],
            program_id,
        )
    }

    /// Get program-derived account address for the conversation
    pub fn find_pda_address(
        first_user_pda_address: &Pubkey,
        second_user_pda_address: &Pubkey,
        program_id: &Pubkey,
    ) -> Pubkey {
        Self::find_pda_address_with_bump_seed(
            first_user_pda_address,
            second_user_pda_address,
            program_id,
        ).0
    }
}

/// Define a user-conversation account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserConversation {
    /// Address of the conversation account
    pub conversation_address: Pubkey,
}

impl UserConversation {
    pub const ACCOUNT_ADDRESS_SEED: &'static str = "user-conversation";

    /// Get size of user-conversation account
    pub fn retrieve_size() -> usize {
        mem::size_of::<UserConversation>()
    }

    /// Get program-derived account address and bump seeds for the user-conversation
    pub fn find_pda_address_with_bump_seed(
        user_pda_address: &Pubkey,
        conversation_index: u32,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &user_pda_address.to_bytes(),
                (conversation_index.to_string() + Self::ACCOUNT_ADDRESS_SEED).as_bytes(),
            ],
            program_id,
        )
    }

    /// Get program-derived account address for the user-conversation
    pub fn find_pda_address(
        user_pda_address: &Pubkey,
        conversation_index: u32,
        program_id: &Pubkey,
    ) -> Pubkey {
        Self::find_pda_address_with_bump_seed(user_pda_address, conversation_index, program_id).0
    }
}

/// Define a message account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Message {
    /// The sender of a message
    pub sender: Pubkey,
    /// Defines content type of an message
    pub message_type: u8,
    /// Message content - depends on message type
    pub content: Vec<u8>,
    /// Timestamp at which user sent a message
    pub timestamp: UnixTimestamp,
}

impl Message {
    pub const ACCOUNT_ADDRESS_SEED: &'static str = "message";

    /// Create a new dummy message account
    pub fn new(content_size: usize) -> Self {
        Self {
            sender: Pubkey::default(),
            message_type: 0,
            content: vec![0_u8; content_size],
            timestamp: UnixTimestamp::default(),
        }
    }

    /// Get size of message account
    pub fn retrieve_size(content_size: usize) -> usize {
        Self::new(content_size).try_to_vec().unwrap().len()
    }

    /// Get program-derived account address and bump seeds for the conversation message
    pub fn find_pda_address_with_bump_seed(
        conversation_address: &Pubkey,
        message_index: u32,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &conversation_address.to_bytes(),
                (message_index.to_string() + Self::ACCOUNT_ADDRESS_SEED).as_bytes(),
            ],
            program_id,
        )
    }

    /// Get program-derived account address for the user-conversation
    pub fn find_pda_address(
        conversation_address: &Pubkey,
        message_index: u32,
        program_id: &Pubkey,
    ) -> Pubkey {
        Self::find_pda_address_with_bump_seed(conversation_address, message_index, program_id).0
    }
}

/// Define message types
#[non_exhaustive]
#[derive(Debug)]
pub struct MessageType;

impl MessageType {
    pub const PLAIN_TEXT: u8 = 0;
    pub const RSA_ENCRYPTED: u8 = 1;
    pub const ARWEAVE: u8 = 2;
}

/// Sort the addresses in ascending order
pub fn sort_addresses_asc<'a>(address_one: &'a Pubkey, address_two: &'a Pubkey) -> [&'a Pubkey; 2] {
    match *address_one < *address_two {
        true => [address_one, address_two],
        false => [address_two, address_one],
    }
}

/// Define a conversation-encryption-info account structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ConversationEncryptionInfo {
    /// Conversation encryption data - it holds info related to encryption
    pub data: Vec<u8>,
}

impl ConversationEncryptionInfo {
    pub const ACCOUNT_ADDRESS_SEED: &'static str = "conversation-encryption";

    /// Create a new dummy conversation-encryption-info account
    pub fn new(data_size: usize) -> Self {
        Self {
            data: vec![0_u8; data_size],
        }
    }

    /// Get size of conversation-encryption-info account
    pub fn retrieve_size(data_size: usize) -> usize {
        Self::new(data_size).try_to_vec().unwrap().len()
    }

    /// Get program-derived account address and bump seeds for the conversation-encryption-info
    pub fn find_pda_address_with_bump_seed(
        conversation_address: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                &conversation_address.to_bytes(),
                Self::ACCOUNT_ADDRESS_SEED.as_bytes(),
            ],
            program_id,
        )
    }

    /// Get program-derived account address for the conversation-encryption-info
    pub fn find_pda_address(
        conversation_address: &Pubkey,
        program_id: &Pubkey,
    ) -> Pubkey {
        Self::find_pda_address_with_bump_seed(conversation_address, program_id).0
    }
}