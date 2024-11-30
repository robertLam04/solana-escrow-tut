import { serialize, Schema } from "borsh";
import { Buffer } from "buffer";

export class Exchange {
    tag: number;
    amount: bigint;

    constructor(fields: { amount: bigint }) {
        this.tag = 1;
        this.amount = fields.amount;
    }
}

// Define the schema for serialization
export const ExchangeSchema: Schema = new Map([
    [
        Exchange,
        { 
            kind: "struct",
            fields: [
                ["tag", "u8"],
                ["amount", "u64"]
            ] 
        }
    ],
]);

export const serializeExchange = (amount: bigint): Buffer => {
    const initEscrow = new Exchange({ amount });
    return Buffer.from(serialize(ExchangeSchema, initEscrow));
};
