import "dotenv/config";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const cluster = process.argv[2] ?? "devnet";
const wallet = resolve(process.env.HOUSE_KEYPAIR ?? process.env.ANCHOR_WALLET ?? "~/.config/solana/id.json");
const solanaUrl = cluster === "local" ? "http://127.0.0.1:8899" : `https://api.${cluster}.solana.com`;

const result = spawnSync(
  "solana",
  [
    "program",
    "deploy",
    "target/deploy/solana_coinflip.so",
    "--keypair",
    wallet,
    "--url",
    solanaUrl
  ],
  { stdio: "inherit" }
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

console.log(`\nProgram deployed. Authority wallet: ${wallet}`);
