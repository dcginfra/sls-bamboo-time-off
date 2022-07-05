use slack_hook3::{PayloadBuilder, Slack};

use sls_bamboo_time_off::bamboo::TimeOffRequest;
use sls_bamboo_time_off::get_required_env_var;

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + Send + Sync)>> {
    let slack_webhook_url = get_required_env_var("SLACK_WEBHOOK_URL")?;
    dbg!(&slack_webhook_url);

    let sample_time_off_request = TimeOffRequest {
        id: 9001,
        name: "Santa Claus".into(),
        created_date: chrono::NaiveDate::from_ymd(2021, 12, 26),
        start_date: chrono::NaiveDate::from_ymd(2021, 12, 26),
        end_date: chrono::NaiveDate::from_ymd(2022, 12, 23),
    };
    dbg!(&sample_time_off_request);

    let slack = Slack::new(slack_webhook_url)?;
    let msg = sample_time_off_request.format_slack_msg();
    let payload = PayloadBuilder::new().text(msg).build()?;

    slack.send(&payload).await?;
    println!("... sent!");

    Ok(())
}
