use aws_lambda_events::dynamodb::Event;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use slack_hook3::{PayloadBuilder, Slack};

use sls_bamboo_time_off::bamboo::TimeOffRequest;
use sls_bamboo_time_off::get_required_env_var;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    // to run locally, comment the above lines and uncomment these below

    // handler(LambdaEvent::new(
    //     Value::Null,
    //     lambda_runtime::Context::default(),
    // ))
    // .await?;

    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    dbg!(&event);

    let slack_alert_cap = get_required_env_var("SLACK_ALERT_CAP")?;
    let slack_notify_enabled = get_required_env_var("SLACK_NOTIFY_ENABLED")?;
    let slack_webhook_url = get_required_env_var("SLACK_WEBHOOK_URL")?;

    dbg!(&slack_alert_cap);
    dbg!(&slack_notify_enabled);
    dbg!(&slack_webhook_url);

    // prevent Slack channel spam
    let max_alert_cap = get_max_alert_cap(&slack_alert_cap);
    dbg!(&max_alert_cap);

    // The Event stream event handled to Lambda
    // http://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-ddb-update
    //

    // to run locally, also uncomment these 2 lines below
    // let sample_json_event = tokio::fs::read_to_string("data/sample-bamboohr.json").await?;
    // let event: Event = serde_json::from_str(&sample_json_event)?;

    let event: Event = serde_json::from_value(event)?;
    dbg!(&event);
    let mut time_off_requests: Vec<TimeOffRequest> = event
        .records
        .iter()
        .filter(|rec| rec.event_name == "INSERT")
        .map(|rec| serde_dynamo::from_item(rec.change.new_image.clone()).unwrap())
        .collect();
    dbg!(&time_off_requests);

    let mut count_alerts = time_off_requests.len();
    println!("Got {} alert(s) to send", count_alerts);

    // skip further processing if this env var not set to exactly "TRUE"
    if slack_notify_enabled != "TRUE" {
        println!("Slack notify disabled, not sending anything. Short-circuiting now.");
        return Ok(json!({
            "message": format!("Did NOT send {} alerts", count_alerts)
        }));
    }

    // prevent Slack channel spam, limit amount of alerts if max alerts reached
    if count_alerts >= max_alert_cap {
        println!(
            "Slack max alert cap reached, limiting number of alerts to {}.",
            max_alert_cap
        );
        time_off_requests = time_off_requests[0..max_alert_cap].to_vec();
        count_alerts = max_alert_cap;
    }

    let slack = Slack::new(slack_webhook_url)?;
    for req in time_off_requests.iter() {
        let msg = req.format_slack_msg();
        dbg!(&msg);
        let payload = PayloadBuilder::new().text(msg).build()?;
        slack.send(&payload).await?;
    }

    Ok(json!({
        "message": format!("Sent {} alerts", count_alerts)
    }))
}

// get_max_alert_cap returns the maximum allowed number of Slack messages in one
// run
fn get_max_alert_cap(env_slack_alert_cap: &str) -> usize {
    let default: usize = 20;
    env_slack_alert_cap.parse::<usize>().unwrap_or(default)
}
