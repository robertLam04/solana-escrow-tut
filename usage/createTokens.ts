import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

export async function createToken(
    connection: Connection,
    payer: Keypair,
    mintAuthority: Keypair
): Promise<PublicKey> {
    const tokenMint = await createMint(connection, payer, mintAuthority.publicKey, null, 6);
    console.log(`Created token with mint address: ${tokenMint.toBase58()}`);
    return tokenMint;
}

export async function createTokenAccount(
    connection: Connection,
    payer: Keypair,
    owner: Keypair,
    mint: PublicKey
): Promise<PublicKey> {
    const tokenAccount = await getOrCreateAssociatedTokenAccount(connection, payer, mint, owner.publicKey);
    console.log(`Created token account: ${tokenAccount.address.toBase58()}`);
    return tokenAccount.address;
}

export async function mintTokens(
    connection: Connection,
    payer: Keypair,
    mint: PublicKey,
    tokenAccount: PublicKey,
    mintAuthority: Keypair,
    amount: number
): Promise<void> {
    await mintTo(connection, payer, mint, tokenAccount, mintAuthority, amount);
    console.log(`Minted ${amount} tokens to account: ${tokenAccount.toBase58()}`);
}
