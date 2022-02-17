import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  MetadataProgram,
  Metadata,
} from "@metaplex-foundation/mpl-token-metadata";
import { Example } from "../target/types/example";

describe("example", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Example as Program<Example>;

  const auth = anchor.web3.Keypair.generate();
  let mint: anchor.web3.PublicKey;
  let ata: anchor.web3.PublicKey;
  let player: anchor.web3.PublicKey;
  let meta: anchor.web3.PublicKey;

  it("Is initialized!", async () => {
    await anchor
      .getProvider()
      .connection.confirmTransaction(
        await anchor
          .getProvider()
          .connection.requestAirdrop(auth.publicKey, 10000000000),
        "confirmed"
      );

    const [playerAccount, bump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [auth.publicKey.toBuffer()],
        program.programId
      );
    player = playerAccount;
    await program.rpc.initialize({
      accounts: {
        authority: auth.publicKey,
        playerAccount: playerAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [auth],
    });
    const playerAccnt = await program.account.player.fetch(player);
    console.log(playerAccnt);
  });

  it("minted", async () => {
    const minted = anchor.web3.Keypair.generate();
    mint = minted.publicKey;
    const [metadata, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(anchor.utils.bytes.utf8.encode("metadata")),
        mint.toBuffer(),
      ],
      program.programId
    );
    meta = metadata;
    ata = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      mint,
      auth.publicKey
    );
    const metaplexMetadataAccount = await Metadata.getPDA(mint);
    await program.rpc.mintItem({
      accounts: {
        authority: auth.publicKey,
        player: player,
        nftMint: mint,
        nftToken: ata, // get ata
        nftMetadata: metadata,
        metaplexMetadataAccount: metaplexMetadataAccount,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: MetadataProgram.PUBKEY,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [auth, minted],
    });

    const client = new Token(
      program.provider.connection,
      minted.publicKey,
      TOKEN_PROGRAM_ID,
      auth
    );

    const mintAccount = await client.getAccountInfo(ata);
    console.log(mintAccount.amount.toNumber());
    const playerAccount = await program.account.player.fetch(player);
    console.log(playerAccount);
  });

  it("redeemed", async () => {
    await program.rpc.redeem({
      accounts: {
        nftMint: mint,
        nftToken: ata,
        nftMetadata: meta,
        player: player,
        authority: auth.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [auth],
    });
    const playerAccnt = await program.account.player.fetch(player);
    console.log(playerAccnt);
  });
});
