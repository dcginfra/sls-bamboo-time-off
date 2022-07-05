use aws_sdk_dynamodb::Client;
use chrono::{NaiveDateTime, Utc};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use std::collections::HashSet;

use sls_bamboo_time_off::alertdb::AlertDB;
use sls_bamboo_time_off::bamboo::{Agent, TimeOffRequest};
use sls_bamboo_time_off::get_required_env_var;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (_event, _context) = event.into_parts();

    let bamboo_api_key = get_required_env_var("BAMBOO_API_KEY")?;
    let bamboo_subdomain = get_required_env_var("BAMBOO_SUBDOMAIN")?;
    let table_name = get_required_env_var("TABLE_NAME")?;

    dbg!(&bamboo_api_key);
    dbg!(&bamboo_subdomain);
    dbg!(&table_name);

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let bamboo_agent = Agent::new(&bamboo_api_key, &bamboo_subdomain);
    // dbg!(&bamboo_agent);

    let ts_now = Utc::now().timestamp();
    // dbg!(&ts_now);

    // 2 months ago
    let start_date_ts = ts_now - (86400 * 61);
    let start_date = NaiveDateTime::from_timestamp(start_date_ts, 0).date();
    dbg!(&start_date);

    // 1 year from now
    let end_date_ts = ts_now + (86400 * 365);
    let end_date = NaiveDateTime::from_timestamp(end_date_ts, 0).date();
    dbg!(&end_date);

    let reqs = bamboo_agent
        .get_time_off_requests(start_date, end_date)
        .await?;
    // dbg!(&reqs);
    println!("Got {} reqs from BambooHR", reqs.len());

    let alertdb = AlertDB::new(client, &table_name);
    let existing_items = alertdb.get_all_items().await?;
    // dbg!(&existing_items);
    println!("Got {} existing items in DynamoDB", existing_items.len());

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

    Ok(json!({ "message": "Ran successfully (I hope)" }))
}
