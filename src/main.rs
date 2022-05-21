// Connect to google bigtables using a credentials file

// Download blocks in a given range of slots.  For each block, find vote transactions.  Parse them out.  Write
// out the results in this format:

// slot#  vote_account  voted_on 1|0 (1=success, 0=failure)

use solana_vote_program::vote_instruction::VoteInstruction;
use std::collections::HashMap;
use std::str::FromStr;

struct Args
{
    credentials_path : String,

    first_slot : u64,

    last_slot : u64
}

fn parse_args() -> Result<Args, String>
{
    let mut args = std::env::args();

    args.nth(0);

    // credentials_path is first arg
    let credentials_path = args.nth(0).ok_or("First argument must be credentials file path".to_string())?;

    let first_slot = args
        .nth(0)
        .ok_or("Second argument must be first slot to fetch".to_string())?
        .parse::<u64>()
        .map_err(|e| format!("Second argument must be first slot to fetch: {}", e))?;

    let last_slot = args
        .nth(0)
        .ok_or("Third argument must be last slot to fetch".to_string())?
        .parse::<u64>()
        .map_err(|e| format!("Third argument must be last slot to fetch: {}", e))?;

    Ok(Args { credentials_path, first_slot, last_slot })
}

#[tokio::main]
async fn main()
{
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(-1);
    });

    let vote_program_id = solana_sdk::pubkey::Pubkey::from_str("Vote111111111111111111111111111111111111111").unwrap();

    let ledger_storage =
        solana_storage_bigtable::LedgerStorage::new(true, None, Some(args.credentials_path.clone())).await.unwrap();

    // Keep track of the latest landed vote for each vote account
    let mut vote_account_state = HashMap::<solana_sdk::pubkey::Pubkey, u64>::new();

    // Take 100 at a time
    let mut block = args.first_slot;

    let total = (args.last_slot - args.first_slot) + 1;

    loop {
        if block > args.last_slot {
            break;
        }

        let limit = std::cmp::min(100, (args.last_slot - block) + 1);

        let range : Vec<u64> = (block..(block + limit)).collect();

        for (slot, block) in ledger_storage.get_confirmed_blocks_with_data(&range.as_slice()).await.unwrap() {
            for result in block.transactions.into_iter().map(|meta| match meta {
                solana_transaction_status::TransactionWithStatusMeta::MissingMetadata(_) => {
                    // Can't use tx with missing metadata because can't know if it succeeded
                    eprintln!("Cannot use tx in slot {}", slot);
                    None
                },
                solana_transaction_status::TransactionWithStatusMeta::Complete(tx) => {
                    if tx.meta.status.is_ok() {
                        Some((tx.transaction.signatures, match tx.transaction.message {
                            solana_sdk::message::VersionedMessage::Legacy(message) => message,
                            solana_sdk::message::VersionedMessage::V0(message) => solana_sdk::message::Message {
                                header : message.header,
                                account_keys : message.account_keys,
                                recent_blockhash : message.recent_blockhash,
                                instructions : message.instructions
                            }
                        }))
                    }
                    // Ignore vote tx that failed, none of the slots landed
                    else {
                        None
                    }
                }
            }) {
                if let Some((_signatures, message)) = result {
                    // Look for vote instructions
                    for i in 0..message.instructions.len() {
                        let instruction = &message.instructions[i];
                        if message.account_keys[instruction.program_id_index as usize] == vote_program_id {
                            // Now must parse the data as a vote instruction
                            match solana_sdk::program_utils::limited_deserialize(&instruction.data) {
                                Ok(VoteInstruction::Vote(vote)) | Ok(VoteInstruction::VoteSwitch(vote, _)) => {
                                    // instruction account 0 is the voting vote account
                                    let vote_account_pubkey = message.account_keys[instruction.accounts[0] as usize];
                                    let mut to_insert = match vote_account_state.remove(&vote_account_pubkey) {
                                        Some(existing) => existing,
                                        None => 0_u64
                                    };
                                    // Write the slot and pubkey
                                    print!("{} {}", slot, vote_account_pubkey);
                                    // Write the voted on slots that are newer than to_insert and update to_insert
                                    for voted_on_slot in vote.slots {
                                        if voted_on_slot > to_insert {
                                            print!(" {}", voted_on_slot);
                                            to_insert = voted_on_slot;
                                        }
                                    }
                                    vote_account_state.insert(vote_account_pubkey.clone(), to_insert);
                                    println!("");
                                },
                                _ => ()
                            }
                        }
                    }
                }
            }
        }

        block += limit;

        eprintln!("There are {}/{} slots remaining", (args.last_slot - block), total);
    }
}
