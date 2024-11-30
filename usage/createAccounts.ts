import { Connection, Keypair, TransactionConfirmationStrategy } from "@solana/web3.js";
import { sign } from "crypto";

export async function airdropSOL(
    connection: Connection,
    account: Keypair,
    amount = 3
): Promise<void> {
    const signature = await connection.requestAirdrop(account.publicKey, amount * 1e9);

    const latestBlockHash = await connection.getLatestBlockhash();
    const lastValidBlockHeight = latestBlockHash.lastValidBlockHeight;

    const confirmationStrategy: TransactionConfirmationStrategy = {
        signature: signature,
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: lastValidBlockHeight
    };

    const confirmationResult = await connection.confirmTransaction(confirmationStrategy);

    console.log(`Airdropped ${amount} SOL to ${account.publicKey.toBase58()}`);
}

export async function createAccounts(connection: Connection,): Promise<{
    initializer: Keypair;
    acceptor: Keypair;
}> {
    const initializer = Keypair.generate();
    const acceptor = Keypair.generate();

    console.log("Airdropping SOL to initializer...");
    await airdropSOL(connection, initializer);

    console.log("Airdropping SOL to acceptor...");
    await airdropSOL(connection, acceptor);

    return { initializer, acceptor };
}
