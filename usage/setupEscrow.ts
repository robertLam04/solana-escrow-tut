import { Connection, Keypair, SystemProgram, PublicKey, sendAndConfirmTransaction, SYSVAR_RENT_PUBKEY, Transaction, TransactionInstruction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { serializeInitEscrow } from "./instructions/initEscrow";

export async function setupEscrow(
    connection: Connection,
    initializer: Keypair,
    initializerXTokenAccount: PublicKey,
    initializerYTokenAccount: PublicKey,
    escrowAccount: Keypair,
    amountXToEscrow: number,
    expectedYFromAcceptor: number
): Promise<void> {
    console.log(`Setting up escrow:
        Initializer: ${initializer.publicKey.toBase58()}
        Token X to Escrow: ${amountXToEscrow}
        Expected Token Y: ${expectedYFromAcceptor}`);

    const escrowProgramId = new PublicKey("GjHTk84Pg8rFTLZKdqhdSWsNCJYTMQHPPCgq7CkaFcaU");

    const escrowAccountSize = 105;
    const rent = await connection.getMinimumBalanceForRentExemption(escrowAccountSize);

    // NEED TO TRANSFER X TOKENS TO TEMP ACCOUNT BEFORE SENDING INSTRUCTION

    const createEscrowAccountIx = SystemProgram.createAccount({
        fromPubkey: initializer.publicKey,
        newAccountPubkey: escrowAccount.publicKey,
        lamports: rent,
        space: escrowAccountSize,
        programId: escrowProgramId
    });

    const bigintExpectedYFromAcceptor = BigInt(expectedYFromAcceptor);
    const serializedInstructionData = serializeInitEscrow(bigintExpectedYFromAcceptor);

    const initEscrowIx = new TransactionInstruction({
        programId: escrowProgramId,
        keys: [
            { pubkey: initializer.publicKey, isSigner: true, isWritable: false },
            { pubkey: initializerXTokenAccount, isSigner: false, isWritable: true },
            { pubkey: initializerYTokenAccount, isSigner: false, isWritable: false },
            { pubkey: escrowAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false}
        ],
        data: serializedInstructionData
    })

    const transaction = new Transaction().add(createEscrowAccountIx).add(initEscrowIx);
    const signature = await sendAndConfirmTransaction(connection, transaction, [initializer, escrowAccount]);

}
