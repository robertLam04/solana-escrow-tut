import { serialize, Schema } from "borsh";
import { Buffer } from "buffer";

export class InitEscrow {
    tag: number;
    amount: bigint;

    constructor(fields: { amount: bigint }) {
        this.tag = 0;
        this.amount = fields.amount;
    }
}

// Define the schema for serialization
export const initEscrowSchema: Schema = new Map([
    [
        InitEscrow,
        { 
            kind: "struct",
            fields: [
                ["tag", "u8"],
                ["amount", "u64"]
            ] 
        }
    ],
]);

export const serializeInitEscrow = (amount: bigint): Buffer => {
    const initEscrow = new InitEscrow({ amount });
    return Buffer.from(serialize(initEscrowSchema, initEscrow));
};
