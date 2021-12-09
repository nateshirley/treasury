use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{prelude::*, solana_program};
use std::convert::Into;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const TREASURY_SEED: &[u8] = b"treasury";
const GOVERNOR_SEED: &[u8] = b"governor";

//this is a spec for treasury governance on solana. specifically with point-based nft voting

#[program]
pub mod treasury {
    use super::*;

    pub fn create_governor(ctx: Context<CreateGovernor>, governor_bump: u8) -> ProgramResult {
        ctx.accounts.governor.points_supply = 10;
        ctx.accounts.governor.bump = governor_bump;
        Ok(())
    }

    pub fn create_transaction(
        ctx: Context<CreateTransaction>,
        pid: Pubkey,
        accs: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> ProgramResult {
        ctx.accounts.governor.transactions = ctx.accounts.governor.transactions + 1;

        let tx = &mut ctx.accounts.transaction;
        tx.governor = ctx.accounts.governor.key();
        tx.program_id = pid;
        tx.accounts = accs;
        tx.data = data;
        tx.points = 9;
        tx.did_execute = false;
        tx.index = ctx.accounts.governor.transactions;

        Ok(())
    }

    pub fn execute_transaction(
        ctx: Context<ExecuteTransaction>,
        treasury_bump: u8,
    ) -> ProgramResult {
        if ctx.accounts.transaction.points < 4 {
            panic!();
        }
        // Execute the transaction signed by the treasury.
        let mut ix: Instruction = (&*ctx.accounts.transaction).into();
        ix.accounts = ix
            .accounts
            .iter()
            .map(|acc| {
                let mut acc = acc.clone();
                if &acc.pubkey == ctx.accounts.treasury.key {
                    acc.is_signer = true;
                }
                acc
            })
            .collect();

        let accounts = ctx.remaining_accounts;
        let seeds = &[&TREASURY_SEED[..], &[treasury_bump]];
        solana_program::program::invoke_signed(&ix, accounts, &[seeds])?;

        // Burn the transaction to ensure one time use.
        ctx.accounts.transaction.did_execute = true;

        Ok(())
    }

    pub fn empty(ctx: Context<Empty>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(governor_bump: u8)]
pub struct CreateGovernor<'info> {
    creator: Signer<'info>,
    #[account(
        init,
        seeds = [GOVERNOR_SEED],
        bump = governor_bump,
        payer = creator
    )]
    governor: Account<'info, Governor>,
    system_program: Program<'info, System>,
}
#[account]
#[derive(Default)]
pub struct Governor {
    participants: u64,
    points_supply: u64,
    transactions: u32,
    bump: u8,
}

#[derive(Accounts)]
pub struct CreateTransaction<'info> {
    creator: Signer<'info>,
    #[account(zero)]
    transaction: Account<'info, Transaction>,
    #[account(mut)]
    governor: Account<'info, Governor>,
}

#[derive(Accounts)]
#[instruction(treasury_bump: u8)]
pub struct ExecuteTransaction<'info> {
    #[account(mut)]
    transaction: Account<'info, Transaction>,
    governor: Account<'info, Governor>,
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = treasury_bump,
    )]
    treasury: AccountInfo<'info>,
}

#[account]
#[derive(Default)]
pub struct Transaction {
    // The multisig account this transaction belongs to.
    pub governor: Pubkey,
    // Target program to execute against.
    pub program_id: Pubkey,
    // Accounts requried for the transaction.
    pub accounts: Vec<TransactionAccount>,
    // Instruction data for the transaction.
    pub data: Vec<u8>,
    // signers[index] is true iff multisig.owners[index] signed the transaction.
    //pub signers: Vec<bool>,
    pub points: u64,
    // Boolean ensuring one time execution.
    pub did_execute: bool,
    // Owner set sequence number.
    pub index: u32,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&Transaction> for Instruction {
    fn from(tx: &Transaction) -> Instruction {
        Instruction {
            program_id: tx.program_id,
            accounts: tx.accounts.iter().map(AccountMeta::from).collect(),
            data: tx.data.clone(),
        }
    }
}
impl From<&TransactionAccount> for AccountMeta {
    fn from(account: &TransactionAccount) -> AccountMeta {
        match account.is_writable {
            false => AccountMeta::new_readonly(account.pubkey, account.is_signer),
            true => AccountMeta::new(account.pubkey, account.is_signer),
        }
    }
}
impl From<&AccountMeta> for TransactionAccount {
    fn from(account_meta: &AccountMeta) -> TransactionAccount {
        TransactionAccount {
            pubkey: account_meta.pubkey,
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        }
    }
}

#[derive(Accounts)]
pub struct Empty {}
/*

so you could have something custom for allocating points and making transfers
key ingredients


some things i need to think a lot more about
- how to balance points and ownership in voting
- points are always inflating, which is a bit of a problem
- i don't want you to just lose all your points bc more people are joining in
- it would be good to do some sort of weighting based on percentile or something like that
i guess you could do like radical points

*/
