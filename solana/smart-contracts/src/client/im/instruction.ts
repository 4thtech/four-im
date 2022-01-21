import {Schema, SolanaBorsh} from '../solanaBorsh';

export enum InstantMessagingInstruction {
  CreateUserAccount = 'CreateUserAccount',
  CreateConversationAccount = 'CreateConversationAccount',
  CreateUserConversationAccount = 'CreateUserConversationAccount',
  CreateMessageAccount = 'CreateMessageAccount',
  CreateConversationEncryptionInfoAccount = 'CreateConversationEncryptionInfoAccount',
  SendMessage = 'SendMessage',
}

export class Instruction extends SolanaBorsh {
  constructor(prop: any) {
    const len = prop[prop['instruction']] != null ? prop[prop['instruction']].length : 0;
    const schema: Schema = new Map([
      [
        Instruction,
        {
          kind: 'enum',
          field: 'instruction',
          values: [
            [InstantMessagingInstruction.CreateUserAccount, [len]],
            [InstantMessagingInstruction.CreateConversationAccount, [len]],
            [InstantMessagingInstruction.CreateUserConversationAccount, [len]],
            [InstantMessagingInstruction.CreateMessageAccount, [len]],
            [InstantMessagingInstruction.CreateConversationEncryptionInfoAccount, [len]],
            [InstantMessagingInstruction.SendMessage, [len]],
          ],
        },
      ],
    ]);

    super(schema);
    this.assign(prop);
  }
}

export class InstructionData extends SolanaBorsh {
  static schema: Record<InstantMessagingInstruction, Schema> = {
    [InstantMessagingInstruction.CreateUserAccount]: new Map([
      [
        InstructionData,
        {
          kind: 'struct',
          fields: [],
        },
      ],
    ]),
    [InstantMessagingInstruction.CreateConversationAccount]: new Map([
      [
        InstructionData,
        {
          kind: 'struct',
          fields: [],
        },
      ],
    ]),
    [InstantMessagingInstruction.CreateUserConversationAccount]: new Map([
      [
        InstructionData,
        {
          kind: 'struct',
          fields: [
            ['conversation_index', 'u32'],
          ],
        },
      ],
    ]),
    [InstantMessagingInstruction.CreateMessageAccount]: new Map([
      [
        InstructionData,
        {
          kind: 'struct',
          fields: [
            ['conversation_index', 'u32'],
            ['message_type', 'u8'],
            ['content', ['u8']],
          ],
        },
      ],
    ]),
    [InstantMessagingInstruction.CreateConversationEncryptionInfoAccount]: new Map([
      [
        InstructionData,
        {
          kind: 'struct',
          fields: [
            ['data', ['u8']],
          ],
        },
      ],
    ]),
    [InstantMessagingInstruction.SendMessage]: new Map([
      [
        InstructionData,
        {
          kind: 'struct',
          fields: [
            ['message_type', 'u8'],
            ['content', ['u8']],
          ],
        },
      ],
    ]),
  };

  constructor(instructionType: InstantMessagingInstruction, prop: any) {
    super(InstructionData.schema[instructionType]);
    this.assign(prop);
  }
}
