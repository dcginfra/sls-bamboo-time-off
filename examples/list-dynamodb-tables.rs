use aws_sdk_dynamodb::Client;

// example dynamodb app : https://github.com/awslabs/aws-sdk-rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_config = aws_config::load_from_env().await;

    let client = Client::new(&shared_config);
    let req = client.list_tables().limit(10);
    let resp = req.send().await?;
    println!("Current dynamodb tables: {:?}", resp.table_names);
    dbg!(&resp);

    Ok(())
}
