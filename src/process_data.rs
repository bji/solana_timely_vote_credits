use std::collections::HashMap;

// xxx load validators.app validator data to get data center ID
// xxx don't compute timely credits, just emit the number of votes with
// latencies 1 - 64 for each validator

struct VoteAccount
{
    pub pubkey : String,

    pub total_transactions : u64,

    pub total_vote_credits : u32,

    pub vote_latencies : Vec<u32>
}

// For serde json loading
#[derive(serde::Deserialize)]
struct ValidatorDetails
{
    pub vote_account : Option<String>,

    pub data_center_key : Option<String>
}

fn load_json_file<T : for<'de> serde::de::Deserialize<'de>>(path : &String) -> Option<T>
{
    std::fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map(|file| {
            serde_json::from_reader(std::io::BufReader::new(file))
                .map_err(|e| {
                    eprintln!("Error reading JSON from {}:\n    {}\n", path, e);
                    e
                })
                .ok()
        })
        .map_err(|e| eprintln!("Error opening JSON file {} for read:\n    {}", path, e))
        .ok()
        .unwrap_or(None)
}

fn main()
{
    // Read validators.app file as first argument
    // Map from pubkey to data center id
    let validator_data_centers = {
        let mut args = std::env::args();
        args.nth(0);
        if let Some(validator_info_file) = args.nth(0) {
            let details : Option<Vec<ValidatorDetails>> = load_json_file(&validator_info_file);
            match details {
                Some(details) => details
                    .into_iter()
                    .filter(|d| d.vote_account.is_some() && d.data_center_key.is_some())
                    .map(|d| (d.vote_account.unwrap(), d.data_center_key.unwrap()))
                    .collect(),
                None => std::process::exit(-1)
            }
        }
        else {
            HashMap::<String, String>::new()
        }
    };

    // Read epoch data from stdin
    let mut vote_accounts = HashMap::<String, VoteAccount>::new();

    let stdin = std::io::stdin();

    let mut lines_processed = 0;

    loop {
        let mut line = String::new();

        if stdin.read_line(&mut line).is_err() {
            break;
        }

        line.truncate(line.len() - 1);

        if line.len() == 0 {
            break;
        }

        let mut split : Vec<&str> = line.split(" ").collect();

        let slot = split.remove(0).parse::<u64>().unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(-1);
        });

        let vote_pubkey = split.remove(0);

        let mut to_insert = match vote_accounts.remove(vote_pubkey) {
            Some(to_insert) => to_insert,
            None => VoteAccount {
                pubkey : vote_pubkey.to_string(),
                total_transactions : 0_u64,
                total_vote_credits : 0_u32,
                vote_latencies : vec![0_u32; 64]
            }
        };

        to_insert.total_transactions += 1;

        for voted_slot in split {
            to_insert.total_vote_credits += 1;
            // Latency is number of slots past the "minimum possible vote slot"
            let latency = (slot - 1) -
                voted_slot.parse::<u64>().unwrap_or_else(|e| {
                    eprintln!("{} for [{}]", e, voted_slot);
                    std::process::exit(-1);
                });
            if latency < (to_insert.vote_latencies.len() as u64) {
                to_insert.vote_latencies[latency as usize] += 1;
            }
        }

        vote_accounts.insert(vote_pubkey.to_string(), to_insert);

        lines_processed += 1;

        if (lines_processed % 1000000) == 0 {
            eprintln!("Processed {} lines", lines_processed);
        }
    }

    let vote_accounts : Vec<VoteAccount> = vote_accounts.into_iter().map(|(_, vote_account)| vote_account).collect();

    for va in vote_accounts {
        let mut data_center = validator_data_centers.get(&va.pubkey).unwrap_or(&"_".to_string()).clone();
        data_center.retain(|c| !c.is_whitespace());
        print!("{} {} {} {}", data_center, va.pubkey, va.total_transactions, va.total_vote_credits);
        for latency in va.vote_latencies {
            print!(" {}", latency);
        }
        println!("");
    }
}
