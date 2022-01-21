import * as borsh from 'borsh';
import {Schema} from 'borsh';

export abstract class SolanaBorsh {
  private readonly schema: Schema;

  protected constructor(schema: Schema) {
    this.schema = schema;
  }

  assign(properties: {[key: string]: any}) {
    Object.keys(properties).forEach((key: string) => {
      this[key as keyof this] = properties[key];
    });
  }

  encode(): Buffer {
    return Buffer.from(
      borsh.serialize(
        this.schema,
        this,
      ),
    );
  }

  static decode<T>(schema: Schema, classType: any, buffer: Buffer): T {
    return borsh.deserialize<T>(schema, classType, buffer);
  }
}

export * from 'borsh';
