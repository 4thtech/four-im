import {Schema} from 'borsh';
import {SolanaBorsh} from '../solanaBorsh';
import {PublicKey} from '@solana/web3.js';
import BN from 'bn.js';

// User account
export type UserType = Omit<User, 'assign' | 'encode'>;

export class User extends SolanaBorsh {
  conversation_counter = 0;

  static ACCOUNT_ADDRESS_SEED = 'user';

  static schema: Schema = new Map([
    [
      User,
      {
        kind: 'struct',
        fields: [
          ['conversation_counter', 'u32'],
        ],
      },
    ],
  ]);

  constructor(properties: UserType | undefined = undefined) {
    super(User.schema);

    if (properties) {
      this.assign(properties);
    }
  }

  static async findPdaAddress(
    userAddress: PublicKey,
    programId: PublicKey,
  ): Promise<PublicKey> {
    const publicKeyNonce = await PublicKey.findProgramAddress(
      [
        userAddress.toBuffer(),
        Buffer.from(this.ACCOUNT_ADDRESS_SEED),
      ],
      programId,
    );

    return publicKeyNonce[0];
  }
}

// Conversation account
export type ConversationType = Omit<Document, 'assign' | 'encode'>;

export class Conversation extends SolanaBorsh {
  message_counter = 0;

  static ACCOUNT_ADDRESS_SEED = 'conversation';

  static schema: Schema = new Map([
    [
      Conversation,
      {
        kind: 'struct',
        fields: [
          ['message_counter', 'u32'],
        ],
      },
    ],
  ]);

  constructor(properties: ConversationType) {
    super(Conversation.schema);

    if (properties) {
      this.assign(properties);
    }
  }

  static async findPdaAddress(
      firstUserPdaAddress: PublicKey,
      secondUserPdaAddress: PublicKey,
    programId: PublicKey,
  ): Promise<PublicKey> {
    // Sort first & second address so that they are always in the same order
    const reverse = firstUserPdaAddress.toBuffer().compare(secondUserPdaAddress.toBuffer()) === 1;
    const orderedAddresses = reverse
        ? [secondUserPdaAddress.toBuffer(), firstUserPdaAddress.toBuffer()]
        : [firstUserPdaAddress.toBuffer(), secondUserPdaAddress.toBuffer()];

    const publicKeyNonce = await PublicKey.findProgramAddress(
      [
          ...orderedAddresses,
        Buffer.from(this.ACCOUNT_ADDRESS_SEED),
      ],
      programId,
    );

    return publicKeyNonce[0];
  }
}

// UserConversation account
export class UserConversation extends SolanaBorsh {
  conversation_address: PublicKey | undefined;

  static ACCOUNT_ADDRESS_SEED = 'user-conversation';

  static schema: Schema = new Map([
    [
      UserConversation,
      {
        kind: 'struct',
        fields: [
          ['conversation_address', [32]],
        ],
      },
    ],
  ]);

  constructor(properties: ConversationType) {
    super(UserConversation.schema);

    if (properties) {
      this.assign(properties);
    }
  }

  static async findPdaAddress(
      userPdaAddress: PublicKey,
      conversationIndex: number,
      programId: PublicKey,
  ): Promise<PublicKey> {
    const publicKeyNonce = await PublicKey.findProgramAddress(
        [
          userPdaAddress.toBuffer(),
          Buffer.from(conversationIndex.toString() + this.ACCOUNT_ADDRESS_SEED),
        ],
        programId,
    );

    return publicKeyNonce[0];
  }
}

// Message account
export class Message extends SolanaBorsh {
  sender: PublicKey | undefined;
  message_type: number | undefined;
  content: Uint8Array | undefined;
  timestamp: BN | undefined;

  static ACCOUNT_ADDRESS_SEED = 'message';

  static schema: Schema = new Map([
    [
      Message,
      {
        kind: 'struct',
        fields: [
          ['sender', [32]],
          ['message_type', 'u8'],
          ['content', ['u8']],
          ['timestamp', 'u64'],
        ],
      },
    ],
  ]);

  constructor(properties: ConversationType) {
    super(Message.schema);

    if (properties) {
      this.assign(properties);
    }
  }

  static async findPdaAddress(
      conversationPdaAddress: PublicKey,
      messageIndex: number,
      programId: PublicKey,
  ): Promise<PublicKey> {
    const publicKeyNonce = await PublicKey.findProgramAddress(
        [
          conversationPdaAddress.toBuffer(),
          Buffer.from(messageIndex.toString() + this.ACCOUNT_ADDRESS_SEED),
        ],
        programId,
    );

    return publicKeyNonce[0];
  }
}

// ConversationEncryptionInfo account
export class ConversationEncryptionInfo extends SolanaBorsh {
  data: Uint8Array | undefined;

  static ACCOUNT_ADDRESS_SEED = 'conversation-encryption';

  static schema: Schema = new Map([
    [
      ConversationEncryptionInfo,
      {
        kind: 'struct',
        fields: [
          ['data', ['u8']],
        ],
      },
    ],
  ]);

  constructor(properties: ConversationType) {
    super(ConversationEncryptionInfo.schema);

    if (properties) {
      this.assign(properties);
    }
  }

  static async findPdaAddress(
      conversationPdaAddress: PublicKey,
      programId: PublicKey,
  ): Promise<PublicKey> {
    const publicKeyNonce = await PublicKey.findProgramAddress(
        [
          conversationPdaAddress.toBuffer(),
          Buffer.from(this.ACCOUNT_ADDRESS_SEED),
        ],
        programId,
    );

    return publicKeyNonce[0];
  }
}

export enum MessageType {
  PLAIN_TEXT = 0,
  RSA_ENCRYPTED = 1,
  ARWEAVE = 2,
}
