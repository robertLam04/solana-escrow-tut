import { Connection, Keypair } from "@solana/web3.js";
import { createAccounts } from "./createAccounts";
import { createToken, createTokenAccount, mintTokens } from "./createTokens";
import { setupEscrow } from "./setupEscrow";
import { makeExchange } from "./makeExchange";

(async () => {

    const connection = new Connection("http://localhost:8899", "confirmed");

    console.log("=== Setting up accounts and tokens ===");

    // Create accounts
    const { initializer, acceptor } = await createAccounts(connection);
    console.log(`Accounts created:\n- Initializer: ${initializer.publicKey.toBase58()}\n- Acceptor: ${acceptor.publicKey.toBase58()}`);
    const escrowAccount = Keypair.generate();

    // Generate mint authority
    const mintAuthority = Keypair.generate();
    console.log(`Mint authority: ${mintAuthority.publicKey.toBase58()}`);

    // Log balances before creating tokens
    const initializerBalance = await connection.getBalance(initializer.publicKey);
    const acceptorBalance = await connection.getBalance(acceptor.publicKey);
    console.log(`Initializer balance: ${initializerBalance / 1e9} SOL`);
    console.log(`Acceptor balance: ${acceptorBalance / 1e9} SOL`);

    // Create tokens
    const tokenX = await createToken(connection,initializer, mintAuthority);
    const tokenY = await createToken(connection, acceptor, mintAuthority);
    console.log(`Tokens created:\n- Token X Mint: ${tokenX.toBase58()}\n- Token Y Mint: ${tokenY.toBase58()}`);

    console.log("\n=== Creating token accounts ===");

    // Initializer's token accounts
    const initializerXTokenAccount = await createTokenAccount(connection, initializer, initializer, tokenX);
    const initializerYTokenAccount = await createTokenAccount(connection, initializer, initializer, tokenY);

    // Acceptor's token accounts
    const acceptorXTokenAccount = await createTokenAccount(connection, acceptor, acceptor, tokenX);
    const acceptorYTokenAccount = await createTokenAccount(connection, acceptor, acceptor, tokenY);

    console.log(`Initializer's token accounts:\n- X Token Account: ${initializerXTokenAccount.toBase58()}\n- Y Token Account: ${initializerYTokenAccount.toBase58()}`);
    console.log(`Acceptor's token accounts:\n- X Token Account: ${acceptorXTokenAccount.toBase58()}\n- Y Token Account: ${acceptorYTokenAccount.toBase58()}`);

    console.log("\n=== Minting tokens ===");

    // Mint tokens
    await mintTokens(connection, initializer, tokenX, initializerXTokenAccount, mintAuthority, 50); // Initializer gets Token X
    await mintTokens(connection, acceptor, tokenY, acceptorYTokenAccount, mintAuthority, 30);      // Acceptor gets Token Y

    console.log("\n=== Setting up escrow ===");

    // Set up escrow
    await setupEscrow(
        connection,
        initializer,
        initializerXTokenAccount,
        initializerYTokenAccount,
        escrowAccount,
        50, // Sending 500 X tokens
        30  // Expecting 300 Y tokens
    );
    console.log("Escrow setup complete:\n- Initializer sending 50 X tokens\n- Expecting 30 Y tokens in return");

    await makeExchange(
        connection,
        acceptor,
        acceptorYTokenAccount,
        acceptorXTokenAccount,
        initializerXTokenAccount,
        initializer.publicKey,
        initializerYTokenAccount,
        escrowAccount,
        50
    )

    console.log("\n=== All steps completed successfully! ===");
})();
