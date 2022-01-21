import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
  TransactionSignature,
} from '@solana/web3.js';
import {
  Conversation,
  ConversationEncryptionInfo,
  Message,
  User,
  UserConversation
} from './state';
import {
  InstantMessagingInstruction,
  Instruction,
  InstructionData,
} from './instruction';
import BN from 'bn.js';

export type SolConversation = {
  index: number;
  address: string;
  messages: SolMessage[];
};

export type SolMessage = {
  index: number,
  sender: string,
  messageType: number,
  content: string,
  timestamp: string,
};

type EncryptionInfoData = {
  type: EncryptionType;
  data: {
    addresses?: string[];
  };
};

enum EncryptionType {
  RSA = 0,
}

export class Service {
  connection: Connection;

  programId: PublicKey;

  payer: Keypair;

  constructor(connection: Connection, programId: PublicKey, payer: Keypair) {
    this.connection = connection;
    this.programId = programId;
    this.payer = payer;
  }

  public async sendMessage(
    receiverWalletAddress: PublicKey,
    messageType: number,
    content: Uint8Array,
  ): Promise<TransactionSignature | void> {
    console.log('Send message to', receiverWalletAddress.toBase58());

    const sender = this.payer;
    const senderWalletAddress = sender.publicKey;

    // Create user PDA account - sender
    const senderPdaAddress = await this.createUserPdaAccount(senderWalletAddress);
    console.log('Sender user PDA account:', senderPdaAddress.toBase58());

    // Create user PDA account - receiver
    const receiverPdaAddress = await this.createUserPdaAccount(receiverWalletAddress);
    console.log('Receiver user PDA account:', receiverPdaAddress.toBase58());

    // Create conversation PDA account
    const conversationPdaAddress = await this.createConversationPdaAccount(senderPdaAddress, receiverPdaAddress);
    console.log('Conversation PDA account:', conversationPdaAddress.toBase58());

    // Create conversation-encryption-info PDA account
    const encryptionInfoData = Buffer.from(JSON.stringify({
      type: EncryptionType.RSA,
      data: {
        addresses: [
          senderWalletAddress.toBase58(),
          receiverWalletAddress.toBase58(),
        ],
      },
    }));
    const conversationEncryptionInfoPdaAddress = await this.createConversationEncryptinoInfoPdaAccount(senderWalletAddress, senderPdaAddress, receiverPdaAddress, conversationPdaAddress, encryptionInfoData);
    console.log('Conversation encryption info PDA account:', conversationEncryptionInfoPdaAddress.toBase58());

    const d = await this.getConversationEncryptionInfoData(conversationPdaAddress);
    console.log(d);


    // Get message PDA address
    const messageIndex = await this.getMessageCounter(conversationPdaAddress);
    const messagePdaAddress = await this.getMessagePdaAddress(conversationPdaAddress, messageIndex);
    console.log('Message PDA account:', messagePdaAddress.toBase58());

    // Send transaction
    const instructionData = new InstructionData(InstantMessagingInstruction.SendMessage, {
      messageType: messageType,
      content,
    }).encode();
    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: this.payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: sender.publicKey, isSigner: true, isWritable: true},
        {pubkey: senderPdaAddress, isSigner: false, isWritable: true},
        {pubkey: receiverPdaAddress, isSigner: false, isWritable: true},
        {pubkey: conversationPdaAddress, isSigner: false, isWritable: true},
        {pubkey: messagePdaAddress, isSigner: false, isWritable: true},
        {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
      ],
      programId: this.programId,
      data: new Instruction({
        instruction: InstantMessagingInstruction.SendMessage,
        [InstantMessagingInstruction.SendMessage]: new Uint8Array(instructionData),
      }).encode(),
    });

    const txSignature = await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(instruction),
      [this.payer],
    );

    return txSignature;
  }

  public async appendMessageToConversation(
    conversationPdaAddress: PublicKey,
    conversationIndex: number,
    messageType: number,
    content: Uint8Array,
  ) {
    return this.createMessagePdaAccount(
      conversationPdaAddress,
      conversationIndex,
      messageType,
      content,
    );
  }

  private async createMessagePdaAccount(
    conversationPdaAddress: PublicKey,
    conversationIndex: number,
    messageType: number,
    content: Uint8Array,
  ) {
    const sender = this.payer;
    const senderWalletAddress = sender.publicKey;

    // Create user PDA account - sender
    const senderPdaAddress = await this.createUserPdaAccount(senderWalletAddress);
    console.log('Sender user PDA account:', senderPdaAddress.toBase58());

    // Get sender user-conversation PDA Address
    const senderUserConversationPdaAddress = await this.getUserConversationPdaAddress(senderPdaAddress, conversationIndex);
    console.log('Sender user-conversation PDA account:', senderUserConversationPdaAddress.toBase58())

    // Get message PDA address
    const messageIndex = await this.getMessageCounter(conversationPdaAddress);
    const messagePdaAddress = await this.getMessagePdaAddress(conversationPdaAddress, messageIndex);
    console.log('Message PDA account:', messagePdaAddress.toBase58());

    // Send transaction
    const instructionData = new InstructionData(InstantMessagingInstruction.CreateMessageAccount, {
      conversation_index: conversationIndex,
      messageType: messageType,
      content,
    }).encode();
    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: this.payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: sender.publicKey, isSigner: true, isWritable: true},
        {pubkey: senderPdaAddress, isSigner: false, isWritable: false},
        {pubkey: senderUserConversationPdaAddress, isSigner: false, isWritable: false},
        {pubkey: conversationPdaAddress, isSigner: false, isWritable: true},
        {pubkey: messagePdaAddress, isSigner: false, isWritable: true},
        {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
      ],
      programId: this.programId,
      data: new Instruction({
        instruction: InstantMessagingInstruction.CreateMessageAccount,
        [InstantMessagingInstruction.CreateMessageAccount]: new Uint8Array(instructionData),
      }).encode(),
    });

    const txSignature = await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(instruction),
      [this.payer],
    );

    return txSignature;
  }

  public async getConversations(userWalletAddress: PublicKey): Promise<SolConversation[]> {
    console.log('Retrieve conversations for', userWalletAddress.toBase58());

    // Get receiver PDA account address
    const userPdaAddress = await User.findPdaAddress(userWalletAddress, this.programId);

    // Get number of conversations
    const conversationCounter = await this.getConversationCounter(userPdaAddress);

    console.log('-> Conversations count:', conversationCounter);

    const conversations = [];

    // Get documents
    for (let i = 0; i < conversationCounter; i++) {
      const userConversationPdaAddress = await this.getUserConversationPdaAddress(userPdaAddress, i);
      const conversationPdaAddress = await this.getConversationAddress(userConversationPdaAddress);
      const messages = await this.getMessages(conversationPdaAddress);

      console.log(messages);

      conversations.push({
        index: i,
        address: conversationPdaAddress.toBase58(),
        messages: messages
      });
    }

    return conversations;
  }

  public async getMessages(conversationPdaAddress: PublicKey): Promise<Array<SolMessage>> {
    console.log('Retrieve messages for conversation', conversationPdaAddress.toBase58());

    // Get number of messages
    const messagesCounter = await this.getMessageCounter(conversationPdaAddress);

    console.log('-> Messages count:', messagesCounter);

    const messages: Array<SolMessage> = [];

    // Get messages
    for (let i = 0; i < messagesCounter; i++) {
      const messagePdaAddress = await Message.findPdaAddress(conversationPdaAddress, i, this.programId);
      const message = await this.getMessage(messagePdaAddress);
      message.index = i;
      messages.push(message);
    }

    return messages;
  }

  private async getMessage(messagePdaAddress: PublicKey) {
    const accountInfo = await this.connection.getAccountInfo(messagePdaAddress);

    if (accountInfo === null) {
      throw Error('Cannot find the message account');
    }

    const message = Message.decode<Message>(Message.schema, Message, accountInfo.data);

    if (!message) {
      throw Error('Problem with message data');
    }

    const {sender, message_type, content, timestamp} = message;

    // TODO: make better deserialization...
    return {
      index: 0,
      sender: new PublicKey(Buffer.from(sender ?? '')).toBase58(),
      messageType: 0,//parseInt(message_type.toString()),
      content: Buffer.from(content ?? '').toString(),
      timestamp: new BN(timestamp ?? 0).toString(),
    }
  }

  private async isAccountExists(pdaAddress: PublicKey): Promise<boolean> {
    return (await this.connection.getAccountInfo(pdaAddress)) !== null;
  }

  private async createUserPdaAccount(userWalletAddress: PublicKey): Promise<PublicKey> {
    // Get user PDA account address
    const userPdaAddress = await User.findPdaAddress(userWalletAddress, this.programId);

    // Return account address if already exist
    if (await this.isAccountExists(userPdaAddress)) {
      return userPdaAddress;
    }

    // Send transaction
    const instructionData = new InstructionData(InstantMessagingInstruction.CreateUserAccount, {}).encode();
    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: this.payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: userPdaAddress, isSigner: false, isWritable: true},
        {pubkey: userWalletAddress, isSigner: false, isWritable: false},
        {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
      ],
      programId: this.programId,
      data: new Instruction({
        instruction: InstantMessagingInstruction.CreateUserAccount,
        [InstantMessagingInstruction.CreateUserAccount]: new Uint8Array(instructionData),
      }).encode(),
    });

    await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(instruction),
      [this.payer],
    );

    return userPdaAddress;
  }

  private async createConversationPdaAccount(senderPdaAddress: PublicKey, receiverPdaAddress: PublicKey): Promise<PublicKey> {
    // Get conversation PDA account address
    const conversationPdaAddress = await Conversation.findPdaAddress(senderPdaAddress, receiverPdaAddress, this.programId);

    // Return account address if already exist
    if (await this.isAccountExists(conversationPdaAddress)) {
      return conversationPdaAddress;
    }

    // Get User conversation counter
    const senderConversationCounter = await this.getConversationCounter(senderPdaAddress);
    const receiverConversationCounter = await this.getConversationCounter(receiverPdaAddress);

    // Get sender user-conversation PDA Address
    const senderUserConversationPdaAddress = await this.getUserConversationPdaAddress(senderPdaAddress, senderConversationCounter);
    console.log('Sender user-conversation PDA account:', senderUserConversationPdaAddress.toBase58())

    // Get receiver user-conversation PDA Address
    const receiverUserConversationPdaAddress = await this.getUserConversationPdaAddress(receiverPdaAddress, receiverConversationCounter);
    console.log('Sender user-conversation PDA account:', receiverUserConversationPdaAddress.toBase58())

    // Send transaction
    const instructionData = new InstructionData(InstantMessagingInstruction.CreateConversationAccount, {}).encode();
    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: this.payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: conversationPdaAddress, isSigner: false, isWritable: true},
        {pubkey: senderPdaAddress, isSigner: false, isWritable: true},
        {pubkey: receiverPdaAddress, isSigner: false, isWritable: true},
        {
          pubkey: senderUserConversationPdaAddress,
          isSigner: false,
          isWritable: true
        },
        {
          pubkey: receiverUserConversationPdaAddress,
          isSigner: false,
          isWritable: true
        },
        {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
      ],
      programId: this.programId,
      data: new Instruction({
        instruction: InstantMessagingInstruction.CreateConversationAccount,
        [InstantMessagingInstruction.CreateConversationAccount]: new Uint8Array(instructionData),
      }).encode(),
    });

    await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(instruction),
      [this.payer],
    );

    return conversationPdaAddress;
  }

  private async createConversationEncryptinoInfoPdaAccount(senderWalletAddress: PublicKey, senderPdaAddress: PublicKey, receiverPdaAddress: PublicKey, conversationPdaAddress: PublicKey, data: Uint8Array): Promise<PublicKey> {
    // Get conversation encryption info PDA account address
    const conversationEncryptionInfoPdaAddress = await ConversationEncryptionInfo.findPdaAddress(conversationPdaAddress, this.programId);

    // Return account address if already exist
    if (await this.isAccountExists(conversationEncryptionInfoPdaAddress)) {
      return conversationEncryptionInfoPdaAddress;
    }

    // Send transaction
    const instructionData = new InstructionData(InstantMessagingInstruction.CreateConversationEncryptionInfoAccount, {
      data,
    }).encode();
    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: this.payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: senderWalletAddress, isSigner: true, isWritable: true},
        {
          pubkey: conversationEncryptionInfoPdaAddress,
          isSigner: false,
          isWritable: true
        },
        {pubkey: senderPdaAddress, isSigner: false, isWritable: false},
        {pubkey: receiverPdaAddress, isSigner: false, isWritable: false},
        {pubkey: conversationPdaAddress, isSigner: false, isWritable: false},
        {pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false},
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
      ],
      programId: this.programId,
      data: new Instruction({
        instruction: InstantMessagingInstruction.CreateConversationEncryptionInfoAccount,
        [InstantMessagingInstruction.CreateConversationEncryptionInfoAccount]: new Uint8Array(instructionData),
      }).encode(),
    });

    await sendAndConfirmTransaction(
      this.connection,
      new Transaction().add(instruction),
      [this.payer],
    );

    return conversationEncryptionInfoPdaAddress;
  }

  private async getUserConversationPdaAddress(userPdaAddress: PublicKey, conversationIndex: number): Promise<PublicKey> {
    return await UserConversation.findPdaAddress(userPdaAddress, conversationIndex, this.programId);
  }

  private async getConversationAddress(userConversationPdaAddress: PublicKey): Promise<PublicKey> {
    const accountInfo = await this.connection.getAccountInfo(userConversationPdaAddress);

    if (accountInfo === null) {
      throw Error('Cannot find the user account');
    }

    const address = UserConversation.decode<UserConversation>(UserConversation.schema, UserConversation, accountInfo?.data)
      .conversation_address;

    return new PublicKey(Buffer.from(address ?? ''));
  }

  private async getMessagePdaAddress(conversationPdaAddress: PublicKey, messageIndex: number): Promise<PublicKey> {
    return await Message.findPdaAddress(conversationPdaAddress, messageIndex, this.programId);
  }

  private async getConversationCounter(userPdaAddress: PublicKey): Promise<number> {
    const accountInfo = await this.connection.getAccountInfo(userPdaAddress);

    if (accountInfo === null) {
      throw Error('Cannot find the user account');
    }

    return User.decode<User>(User.schema, User, accountInfo.data)
      .conversation_counter;
  }

  private async getMessageCounter(conversationPdaAddress: PublicKey): Promise<number> {
    const accountInfo = await this.connection.getAccountInfo(conversationPdaAddress);

    if (accountInfo === null) {
      throw Error('Cannot find the conversation account');
    }

    return Conversation.decode<Conversation>(Conversation.schema, Conversation, accountInfo?.data)
      .message_counter;
  }

  private async getConversationEncryptionInfoData(conversationPdaAddress: PublicKey): Promise<EncryptionInfoData> {
    // Get conversation encryption info PDA account address
    const conversationEncryptionInfoPdaAddress = await ConversationEncryptionInfo.findPdaAddress(conversationPdaAddress, this.programId);

    const accountInfo = await this.connection.getAccountInfo(conversationEncryptionInfoPdaAddress);

    if (accountInfo === null) {
      throw Error('Cannot find the conversation account');
    }

    const encryptionInfoData = ConversationEncryptionInfo.decode<ConversationEncryptionInfo>(ConversationEncryptionInfo.schema, ConversationEncryptionInfo, accountInfo.data);

    if (!encryptionInfoData) {
      throw Error('Problem with encryptionInfoData');
    }

    return JSON.parse(
      Buffer.from(encryptionInfoData.data ?? '').toString(),
    );
  }
}
