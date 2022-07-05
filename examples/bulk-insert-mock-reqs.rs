use aws_sdk_dynamodb::Client;
use std::collections::HashSet;
use std::{env, process};

use sls_bamboo_time_off::alertdb::AlertDB;
use sls_bamboo_time_off::bamboo::{Agent, TimeOffRequest};

// time-off-alerts-table

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + Send + Sync)>> {
    let table_name = env::var("TABLE_NAME").unwrap_or_else(|_| {
        println!("error: required env var TABLE_NAME is not set");
        process::exit(1);
    });

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let alertdb = AlertDB::new(client, &table_name);
    // dbg!(&alertdb);

    let bamboo_agent = Agent::new("", "");
    let reqs = bamboo_agent.mock_time_off_requests().await?;
    println!("Got {} reqs from BambooHR", reqs.len());

    let existing_items = alertdb.get_all_items().await?;
    println!("Got {} existing items in DynamoDB", existing_items.len());
    // dbg!(&existing_items);

    let mut existing_ids = HashSet::new();
    for item in existing_items.iter() {
        existing_ids.insert(item.id);
    }

    let mut new_items: Vec<TimeOffRequest> = vec![];
    for item in reqs.iter() {
        if !existing_ids.contains(&item.id) {
            new_items.push(item.clone());
        }
    }
    println!("Got {} new items to be inserted", new_items.len());
    dbg!(&new_items);

    alertdb.write_new_items(&new_items).await?;

    Ok(())
}
