use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use solana_program::keccak;

declare_id!("CoinFlip1111111111111111111111111111111111111");

const MIN_WAGER_LAMPORTS: u64 = 100_000; // 0.0001 SOL

#[program]
pub mod solana_coinflip {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.house = ctx.accounts.house.key();
        config.config_bump = *ctx
            .bumps
            .get("config")
            .ok_or(CoinflipError::MissingSeeds)?;
        config.vault_bump = *ctx
            .bumps
            .get("vault")
            .ok_or(CoinflipError::MissingSeeds)?;
        Ok(())
    }

    pub fn fund_vault(ctx: Context<FundVault>, amount: u64) -> Result<()> {
        require!(amount > 0, CoinflipError::InvalidAmount);
        transfer(
            ctx.accounts.transfer_to_vault_ctx(),
            amount,
        )?;
        Ok(())
    }

    pub fn play(ctx: Context<Play>, guess_heads: bool, wager: u64) -> Result<()> {
        require!(wager >= MIN_WAGER_LAMPORTS, CoinflipError::WagerTooSmall);

        // Move wager into the vault
        transfer(
            ctx.accounts.transfer_player_to_vault_ctx(),
            wager,
        )?;

        // Compute deterministic but verifiable pseudo-random outcome
        let clock = Clock::get()?;
        let outcome_heads = coin_outcome(&ctx.accounts.player.key(), clock.slot, clock.unix_timestamp);

        let state = &mut ctx.accounts.player_state;
        if state.owner == Pubkey::default() {
            state.owner = ctx.accounts.player.key();
        }
        require_keys_eq!(state.owner, ctx.accounts.player.key(), CoinflipError::StateOwnershipMismatch);
        if state.bump == 0 {
            if let Some(bump) = ctx.bumps.get("player_state") {
                state.bump = *bump;
            }
        }
        state.played = state
            .played
            .checked_add(1)
            .ok_or(CoinflipError::MathOverflow)?;
        state.last_guess_heads = guess_heads;
        state.last_outcome_heads = outcome_heads;
        state.updated_at = clock.unix_timestamp;

        if outcome_heads == guess_heads {
            let payout = wager
                .checked_mul(2)
                .ok_or(CoinflipError::MathOverflow)?;
            require!(ctx.accounts.vault.to_account_info().lamports() >= payout, CoinflipError::VaultInsufficient);
            state.wins = state
                .wins
                .checked_add(1)
                .ok_or(CoinflipError::MathOverflow)?;
            transfer(
                ctx.accounts.transfer_vault_to_player_ctx(),
                payout,
            )?;
        } else {
            state.losses = state
                .losses
                .checked_add(1)
                .ok_or(CoinflipError::MathOverflow)?;
        }

        Ok(())
    }
}

fn coin_outcome(player: &Pubkey, slot: u64, timestamp: i64) -> bool {
    let mut seed = Vec::with_capacity(32 + 8 + 8);
    seed.extend_from_slice(player.as_ref());
    seed.extend_from_slice(&slot.to_le_bytes());
    seed.extend_from_slice(&timestamp.to_le_bytes());
    let hash = keccak::hash(&seed);
    hash.0[0] & 1 == 1
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub house: Signer<'info>,
    #[account(
        init,
        payer = house,
        space = Config::SIZE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = house,
        seeds = [b"vault", config.key().as_ref()],
        bump,
        space = 0,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundVault<'info> {
    #[account(mut)]
    pub house: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref()],
        bump = config.vault_bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(mut, seeds = [b"config"], bump = config.config_bump, has_one = house)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundVault<'info> {
    fn transfer_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.house.to_account_info(),
            to: self.vault.to_account_info(),
        };
        CpiContext::new(self.system_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Play<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(mut)]
    pub house: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"vault", config.key().as_ref()],
        bump = config.vault_bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = player,
        space = PlayerState::SIZE,
        seeds = [b"state", player.key().as_ref()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,
    #[account(mut, seeds = [b"config"], bump = config.config_bump, has_one = house)]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

impl<'info> Play<'info> {
    fn transfer_player_to_vault_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.player.to_account_info(),
            to: self.vault.to_account_info(),
        };
        CpiContext::new(self.system_program.to_account_info(), cpi_accounts)
    }

    fn transfer_vault_to_player_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let seeds: &[&[u8]] = &[b"vault", self.config.key().as_ref(), &[self.config.vault_bump]];
        let signer = &[seeds];
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.player.to_account_info(),
        };
        CpiContext::new_with_signer(self.system_program.to_account_info(), cpi_accounts, signer)
    }
}

#[account]
pub struct Config {
    pub house: Pubkey,
    pub config_bump: u8,
    pub vault_bump: u8,
}

impl Config {
    pub const SIZE: usize = 8 + 32 + 1 + 1;
}

#[account]
pub struct PlayerState {
    pub owner: Pubkey,
    pub wins: u64,
    pub losses: u64,
    pub played: u64,
    pub last_guess_heads: bool,
    pub last_outcome_heads: bool,
    pub updated_at: i64,
    pub bump: u8,
}

impl PlayerState {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 1 + 1 + 8 + 1;
}

#[error_code]
pub enum CoinflipError {
    #[msg("Wager must be at least the minimum threshold")]
    WagerTooSmall,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Vault does not have enough lamports for payout")]
    VaultInsufficient,
    #[msg("Player state account owner mismatch")]
    StateOwnershipMismatch,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Program derived address seeds missing from context")]
    MissingSeeds,
}
