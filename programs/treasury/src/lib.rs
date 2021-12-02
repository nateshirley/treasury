use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const TRANSFER_SEED: &[u8] = b"transfer";

//this is a spec for implementing a basic treasury contract on solana

#[program]
pub mod treasury {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }

    pub fn queue_transfer(
        ctx: Context<QueueTransfer>,
        transfer_bump: u8,
        recipient: Pubkey,
        lamports: u64,
    ) -> ProgramResult {
        ctx.accounts.transfer.index = ctx.accounts.governing_body.transfers;
        ctx.accounts.governing_body.transfers = ctx.accounts.governing_body.transfers + 1;
        ctx.accounts.transfer.recipient = recipient;
        ctx.accounts.transfer.lamports = lamports;
        ctx.accounts.transfer.bump = transfer_bump;
        Ok(())
    }
}

pub fn _forum_address() -> Pubkey {
    let program_id = Pubkey::default();
    let (address, bump) = Pubkey::find_program_address(&[TRANSFER_SEED], &program_id);
    address
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
#[instruction(transfer_bump: u8)]
pub struct QueueTransfer<'info> {
    initializer: Signer<'info>,
    #[account(
        init,
        seeds = [TRANSFER_SEED],
        bump = transfer_bump,
        payer = initializer,
    )]
    transfer: Account<'info, Transfer>,
    #[account(
        constraint = forum.key() == _forum_address()
    )]
    forum: AccountInfo<'info>,
    governing_body: Account<'info, GoverningBody>,
    system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct Transfer {
    index: u64,
    recipient: Pubkey,
    lamports: u64,
    votes: u64,
    bump: u8,
}

#[account]
pub struct GoverningBody {
    transfers: u64,
    participants: u64,
    points_supply: u64,
}

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
