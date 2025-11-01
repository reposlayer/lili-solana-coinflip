# Solana Coinflip

A production-ready Anchor program and test suite for a provably fair coinflip game. Players wager SOL, choose heads or tails, and the vault automatically pays out winners at 2x their stake. This starter ships with automated bootstrap commands so you can deploy to devnet and iterate fast.

## Features

- Anchor program with PDA-backed vault
- Deterministic pseudo-random outcome derived from slot + timestamp + player seed
- Player statistics account (wins, losses, last outcome)
- TypeScript tests using Anchor's mocha runner
- Deployment script with configurable authority wallet

## Bootstrap

After cloning through the Lili CLI bootstrap flow, the following commands will run automatically:

```bash
npm install
anchor build
anchor deploy
```

## Manual Commands

```bash
# Build program
anchor build

# Run the mocha suite
anchor test

# Deploy to devnet with the house authority wallet
ts-node scripts/deploy.ts devnet
```

Set `HOUSE_KEYPAIR` in `.env` (copy from `.env.example`) to use a different wallet for funding the vault.

## PDA Layout

- `config` – stores house authority and bump seeds
- `vault` – system account holding SOL for payouts
- `state/<player>` – per-player statistics

## Next Steps

- Replace the pseudo-random function with a VRF oracle for stronger fairness
- Build a wallet-adapter powered frontend in a `web/` folder and reuse the on-chain program
- Add leaderboards or wager limits depending on your jurisdiction
