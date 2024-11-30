import { Connection, Keypair, SystemProgram, PublicKey, sendAndConfirmTransaction, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { serializeExchange } from "./instructions/exchange";

export async function makeExchange (
    connection: Connection,
    acceptor: Keypair,
    acceptorYTokenAccount: PublicKey,
    acceptorXTokenAccount: PublicKey,
    pdaTempAccount: PublicKey,
    initializer: PublicKey,
    initializerYTokenAccount: PublicKey,
    escrowAccount: Keypair,
    expectedXFromInitializer: number
): Promise<void> {

    const escrowProgramId = new PublicKey("GjHTk84Pg8rFTLZKdqhdSWsNCJYTMQHPPCgq7CkaFcaU");

    const [pda, bumpSeed] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow")], // Same seed as used in Rust
        escrowProgramId
    );

    const bigintExpectedYFromAcceptor = BigInt(expectedXFromInitializer);
    const serializedData = serializeExchange(bigintExpectedYFromAcceptor);

    const exchangeIx = new TransactionInstruction ({
        programId: escrowProgramId,
        keys: [
            { pubkey: acceptor.publicKey, isSigner: true, isWritable: false },
            { pubkey: acceptorYTokenAccount, isSigner: false, isWritable: true },
            { pubkey: acceptorXTokenAccount, isSigner: false, isWritable: true },
            { pubkey: pdaTempAccount, isSigner: false, isWritable: true },
            { pubkey: initializer, isSigner: false, isWritable: true },
            { pubkey: initializerYTokenAccount, isSigner: false, isWritable: true },
            { pubkey: escrowAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false},
            { pubkey: pda, isSigner: false, isWritable: false}
        ],
        data: serializedData
    });

    const transaction = new Transaction().add(exchangeIx);
    const signature = await sendAndConfirmTransaction(connection, transaction, [acceptor]);

}