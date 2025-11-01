import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { SolanaCoinflip } from "../target/types/solana_coinflip";

describe("solana-coinflip", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaCoinflip as anchor.Program<SolanaCoinflip>;

  const house = provider.wallet;

  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), configPda.toBuffer()],
    program.programId
  );

  const [playerStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("state"), house.publicKey.toBuffer()],
    program.programId
  );

  it("initializes the vault", async () => {
    await program.methods
      .initialize()
      .accounts({
        house: house.publicKey,
        config: configPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId
      })
      .rpc();

    type ConfigAccount = {
      house: PublicKey;
    };

    const config = (await program.account.config.fetch(configPda)) as ConfigAccount;
    expect(config.house.toBase58()).to.equal(house.publicKey.toBase58());
  });

  it("funds and plays a round", async () => {
    const wager = new anchor.BN(200_000);
    const fundAmount = new anchor.BN(2_000_000);

    await program.methods
      .fundVault(fundAmount)
      .accounts({
        house: house.publicKey,
        vault: vaultPda,
        config: configPda,
        systemProgram: SystemProgram.programId
      })
      .rpc();

    await program.methods
      .play(true, wager)
      .accounts({
        player: house.publicKey,
        house: house.publicKey,
        vault: vaultPda,
        playerState: playerStatePda,
        config: configPda,
        systemProgram: SystemProgram.programId
      })
      .rpc();

    type PlayerStateAccount = {
      played: anchor.BN;
      wins: anchor.BN;
      losses: anchor.BN;
    };

    const state = (await program.account.playerState.fetch(playerStatePda)) as PlayerStateAccount;
    expect(state.played.toNumber()).to.equal(1);
    expect(state.wins.toNumber() + state.losses.toNumber()).to.equal(1);
  });
});
