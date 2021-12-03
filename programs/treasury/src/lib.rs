use anchor_lang::{prelude::*, solana_program};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const TRANSFER_SEED: &[u8] = b"transfer";
const EXECUTE_SEED: &[u8] = b"execute";

//this is a spec for implementing a basic treasury contract on solana

#[program]
pub mod treasury {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }

    pub fn execute_instruction(
        ctx: Context<ExecuteInstruction>,
        ix_program_id: Pubkey,
        ix_metas: Vec<AcctMeta>,
        ix_execution_data: Vec<u8>,
        ix_account_key: Pubkey,
    ) -> ProgramResult {
        /*
        what anchor does is take the pubkeys and deserialize them into account infos
        so if i take an array of pubkeys, i can do that as well,
        */

        let mut metas: Vec<AccountMeta> = vec![];
        for account in ix_metas.iter() {
            metas.push(account.to_account_meta())
        }

        let accounts = metas.clone();

        let (key, bump) = Pubkey::find_program_address(&[EXECUTE_SEED], ctx.program_id);
        let seeds = &[&EXECUTE_SEED, &[bump][..]];

        let built_ix = solana_program::instruction::Instruction {
            program_id: ix_program_id,
            accounts: metas,
            data: ix_execution_data,
        };
        solana_program::program::invoke_signed(
            &built_ix,
            &[
                ctx.accounts.from.clone(),
                ctx.accounts.to.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[seeds],
        )?;

        // solana_program::program::invoke_signed(
        //     &solana_program::system_instruction::transfer(
        //         ctx.accounts.from.key,
        //         ctx.accounts.to.key,
        //         50000,
        //     ),
        //     &[
        //         ctx.accounts.from.clone(),
        //         ctx.accounts.to.clone(),
        //         ctx.accounts.system_program.to_account_info(),
        //     ],
        //     &[seeds],
        // )?;
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
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    from: AccountInfo<'info>,
    #[account(mut)]
    to: AccountInfo<'info>,
    system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Default, Clone)]
pub struct InstructionData {
    /// Pubkey of the instruction processor that executes this instruction
    pub program_id: Pubkey,
    /// Metadata for what accounts should be passed to the instruction processor
    pub accounts: Vec<AcctMeta>,
    /// Opaque data passed to the instruction processor
    pub data: Vec<u8>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Default, Clone)]
pub struct AcctMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl InstructionData {
    fn to_account_metas(&self) -> Vec<AccountMeta> {
        let mut metas: Vec<AccountMeta> = vec![];
        for account in self.accounts.iter() {
            metas.push(account.to_account_meta())
        }
        metas
    }
}
impl AcctMeta {
    fn to_account_meta(&self) -> AccountMeta {
        return AccountMeta {
            pubkey: self.pubkey,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        };
    }
}

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
