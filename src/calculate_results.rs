use std::collections::HashMap;

struct Entry
{
    pub name : String,

    pub total_transactions : u64,

    pub total_credits : u64,

    pub total_timely_credits : u64,

    pub total_latency : u64,

    pub total_validators : u64
}

fn main()
{
    let mut args = std::env::args();
    args.nth(0);

    let by_validators = match args
        .nth(0)
        .unwrap_or_else(|| {
            eprintln!("First argument must be \"v\" or \"d\" (for validators or data centers)");
            std::process::exit(-1);
        })
        .as_str()
    {
        "v" => true,
        "d" => false,
        _ => {
            eprintln!("First argument must be \"v\" or \"d\" (for validators or data centers)");
            std::process::exit(-1);
        }
    };
    let grace = args
        .nth(0)
        .unwrap_or_else(|| {
            eprintln!("Second argument must be grace period");
            std::process::exit(-1);
        })
        .parse::<u64>()
        .unwrap_or_else(|e| {
            eprintln!("Second argument must be grace period: {}", e);
            std::process::exit(-1);
        });
    let max_credits = args
        .nth(0)
        .unwrap_or_else(|| {
            eprintln!("Third argument must be max credits");
            std::process::exit(-1);
        })
        .parse::<u64>()
        .unwrap_or_else(|e| {
            eprintln!("Third argument must be max credits: {}", e);
            std::process::exit(-1);
        });
    let multiplier = args
        .nth(0)
        .unwrap_or_else(|| {
            eprintln!("Fourth argument must be multiplier");
            std::process::exit(-1);
        })
        .parse::<f64>()
        .unwrap_or_else(|e| {
            eprintln!("Fourth argument must be multiplier: {}", e);
            std::process::exit(-1);
        });

    let stdin = std::io::stdin();

    let mut data_centers = HashMap::<String, Entry>::new();

    let mut validators = Vec::<Entry>::new();

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

        // data_center
        // vote_account
        // total_transactions
        // total_credits
        // 64 slots
        if split.len() != 68 {
            eprintln!("Invalid input line: {}", line);
            std::process::exit(-1);
        }

        let data_center = split.remove(0);

        let vote_account = split.remove(0);

        let total_transactions = split.remove(0).to_string().parse::<u64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (total transactions {}): {}", e, line);
            std::process::exit(-1);
        });

        let total_credits = split.remove(0).to_string().parse::<u64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (total credits {}): {}", e, line);
            std::process::exit(-1);
        });

        let mut total_latency = 0_u64;

        let mut total_timely_credits = 0_u64;

        for i in 0..split.len() {
            let slots_at_this_latency = split[i].parse::<u64>().unwrap_or_else(|e| {
                eprintln!("Invalid input line ({}): {}", e, line);
                std::process::exit(-1);
            });

            let i = i as u64;

            total_latency += i * slots_at_this_latency;

            let slot_credits_reduction = (((std::cmp::max(i, grace) - grace) as f64) * multiplier) as u64;

            let slot_credits =
                if max_credits > slot_credits_reduction { max_credits - slot_credits_reduction } else { 1 };

            total_timely_credits += slot_credits * slots_at_this_latency;
        }

        validators.push(Entry {
            name : vote_account.to_string(),
            total_transactions,
            total_credits,
            total_timely_credits,
            total_latency,
            total_validators : 1
        });

        let mut to_insert = match data_centers.remove(data_center) {
            Some(to_insert) => to_insert,
            None => Entry {
                name : data_center.to_string(),
                total_transactions : 0_u64,
                total_credits : 0_u64,
                total_timely_credits : 0_u64,
                total_latency : 0_u64,
                total_validators : 0_u64
            }
        };

        to_insert.total_transactions += total_transactions;
        to_insert.total_credits += total_credits;
        to_insert.total_timely_credits += total_timely_credits;
        to_insert.total_latency += total_latency;
        to_insert.total_validators += 1;

        data_centers.insert(data_center.to_string(), to_insert);
    }

    // Now average out the values for the data centers
    for (_, entry) in &mut data_centers {
        entry.total_transactions /= entry.total_validators;
        entry.total_credits /= entry.total_validators;
        entry.total_timely_credits /= entry.total_validators;
        entry.total_latency /= entry.total_validators;
    }

    let entries = if by_validators { validators } else { data_centers.into_iter().map(|(_, entry)| entry).collect() };

    let max_total_credits = entries.iter().map(|e| e.total_credits).max().unwrap() as f64;

    let max_total_timely_credits = entries.iter().map(|e| e.total_timely_credits).max().unwrap() as f64;

    for entry in entries {
        println!(
            "{} {} {} {} {} {} {} {}",
            entry.name,
            entry.total_validators,
            entry.total_transactions,
            entry.total_credits,
            entry.total_timely_credits,
            (entry.total_latency as f64) / (entry.total_credits as f64),
            (entry.total_credits as f64) / max_total_credits,
            (entry.total_timely_credits as f64) / max_total_timely_credits,
        );
    }
}
