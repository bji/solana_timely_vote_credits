// Takes calculated results and emits an HTML table with:
// Ranking (normal credits)
// Icon (nothing for DataCenter)
// Name
// Ranking timely credits
// % normal credits
// % timely credits
// % change

use std::collections::HashMap;

#[derive(Clone)]
struct Entry
{
    pub name : String,

    pub total_validators : u64,

    pub normal_pct : f64,

    pub timely_pct : f64,

    pub avg_latency : f64,

    pub total_normal_credits : u64,

    pub total_timely_credits : u64,

    pub total_epochs : u64
}

// For serde json loading
#[derive(serde::Deserialize)]
struct ValidatorDetails
{
    pub vote_account : Option<String>,

    pub name : Option<String>,

    pub avatar_url : Option<String>
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
    let mut args = std::env::args();
    args.nth(0);

    let of_validators = match args
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

    // Read validators.app file as second argument
    // Map from pubkey to (name, icon)
    let validator_details = {
        if let Some(validator_info_file) = args.nth(0) {
            let details : Option<Vec<ValidatorDetails>> = load_json_file(&validator_info_file);
            match details {
                Some(details) => details
                    .into_iter()
                    .filter_map(|d| {
                        if d.vote_account.is_some() {
                            Some((d.vote_account.clone().unwrap(), (d.name, d.avatar_url)))
                        }
                        else {
                            None
                        }
                    })
                    .collect(),
                None => std::process::exit(-1)
            }
        }
        else {
            HashMap::<String, (Option<String>, Option<String>)>::new()
        }
    };

    // Read data from stdin.  More than one epoch's worth of data may be included, and if so, the results
    // will be an average across all of those epochs.
    let mut normal_entries = HashMap::<String, Entry>::new();

    let stdin = std::io::stdin();

    loop {
        let mut line = String::new();

        if stdin.read_line(&mut line).is_err() {
            break;
        }

        line.truncate(line.len() - 1);

        if line.len() == 0 {
            break;
        }

        let split : Vec<&str> = line.split(" ").collect();

        let mut name = split[0].to_string();

        // Data center names are of the form ASN-COUNTRY-CITY
        // It is desirable to show COUNTRY-CITY first because column sort then allows easy view of diffs by
        // global region ...
        if !of_validators {
            let split : Vec<&str> = name.split("-").collect();
            if split.len() > 2 {
                let mut new_name = format!("{}-{}-{}", split[1], split[2], split[0]);
                if split.len() > 3 {
                    for i in 3..split.len() {
                        new_name.push_str(format!("-{}", split[i]).as_str());
                    }
                }
                name = new_name;
            }
        }

        let total_validators = split[1].to_string().parse::<u64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (total validators {}): {}", e, line);
            std::process::exit(-1);
        });

        let total_normal_credits = split[3].to_string().parse::<u64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (total normal credits {}): {}", e, line);
            std::process::exit(-1);
        });

        let total_timely_credits = split[4].to_string().parse::<u64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (total timely credits {}): {}", e, line);
            std::process::exit(-1);
        });

        let avg_latency = split[5].to_string().parse::<f64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (avg latency {}): {}", e, line);
            std::process::exit(-1);
        });

        let normal_pct = split[6].to_string().parse::<f64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (normal pct {}): {}", e, line);
            std::process::exit(-1);
        });

        let timely_pct = split[7].to_string().parse::<f64>().unwrap_or_else(|e| {
            eprintln!("Invalid input line (timely pct {}): {}", e, line);
            std::process::exit(-1);
        });

        let mut to_insert = match normal_entries.remove(&name) {
            Some(to_insert) => to_insert,
            None => Entry {
                name : name.to_string(),
                total_validators : 0,
                normal_pct : 0_f64,
                timely_pct : 0_f64,
                avg_latency : 0_f64,
                total_normal_credits : 0_u64,
                total_timely_credits : 0_u64,
                total_epochs : 0_u64
            }
        };

        to_insert.total_validators += total_validators;
        to_insert.normal_pct += normal_pct;
        to_insert.timely_pct += timely_pct;
        to_insert.avg_latency += avg_latency * (total_normal_credits as f64);
        to_insert.total_normal_credits += total_normal_credits;
        to_insert.total_timely_credits += total_timely_credits;
        to_insert.total_epochs += 1;

        normal_entries.insert(name.to_string(), to_insert);
    }

    // Now compute average across all epochs for all entries
    normal_entries.iter_mut().for_each(|(_, e)| {
        e.total_validators /= e.total_epochs;
        e.normal_pct /= e.total_epochs as f64;
        e.avg_latency /= e.total_normal_credits as f64;
        e.timely_pct /= e.total_epochs as f64;
    });

    let mut normal_entries : Vec<Entry> = normal_entries.into_iter().map(|(_, e)| e).collect();

    // Sort entries by normal pct and timely pct
    let mut timely_entries = normal_entries.clone();

    normal_entries.sort_by(|a, b| b.normal_pct.partial_cmp(&a.normal_pct).unwrap());

    timely_entries.sort_by(|a, b| b.timely_pct.partial_cmp(&a.timely_pct).unwrap());

    println!(
        "<table class=\"sortable\" border=1><tr><th>Normal Ranking</th><th>TR</th><th>{}</th><th>Name</th><th>Avg \
         Vote Latency</th><th>Normal Pct</th><th>Diff</th><th>Timely \
         Pct</th><th>Name</th><th>{}</th><th>NR</th><th>Timely Ranking</th></tr>",
        if of_validators { "Icon" } else { "Population" },
        if of_validators { "Icon" } else { "Population" }
    );

    // Now make a table which shows side by side
    for i in 0..normal_entries.len() {
        let normal_entry = &normal_entries[i];
        let timely_entry = &timely_entries[i];
        let (normal_name, normal_icon) = match validator_details.get(&normal_entry.name) {
            Some(entry) => {
                let name = entry.0.clone().unwrap_or(normal_entry.name.clone());
                let mut icon =
                    entry.1.clone().unwrap_or("https://www.shinobi-systems.com/missing_icon.png".to_string());
                icon = format!("<img src=\"{}\" width=30 height=30></img>", icon);
                (name, icon)
            },
            None => (normal_entry.name.clone(), "".to_string())
        };
        let (timely_name, timely_icon) = match validator_details.get(&timely_entry.name) {
            Some(entry) => {
                let name = entry.0.clone().unwrap_or(timely_entry.name.clone());
                let mut icon =
                    entry.1.clone().unwrap_or("https://www.shinobi-systems.com/missing_icon.png".to_string());
                icon = format!("<img src=\"{}\" width=30 height=30></img>", icon);
                (name, icon)
            },
            None => (timely_entry.name.clone(), "".to_string())
        };
        let normal_timely_index = timely_entries.iter().position(|e| e.name == normal_entry.name).unwrap();
        let timely_normal_index = normal_entries.iter().position(|e| e.name == timely_entry.name).unwrap();

        println!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:0.5}</td><td>{:0.3}%</td><td>{:0.3}%</td><td>{:0.\
             3}%</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            i + 1,
            normal_timely_index + 1,
            if of_validators { normal_icon } else { normal_entry.total_validators.to_string() },
            normal_name,
            normal_entry.avg_latency,
            normal_entry.normal_pct * 100_f64,
            ((normal_entry.timely_pct - normal_entry.normal_pct) / normal_entry.normal_pct) * 100_f64,
            timely_entry.timely_pct * 100_f64,
            timely_name,
            if of_validators { timely_icon } else { timely_entry.total_validators.to_string() },
            timely_normal_index + 1,
            i + 1
        );
    }

    println!("</table>");
}
