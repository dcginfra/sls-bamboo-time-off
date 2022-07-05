use chrono::NaiveDate;

use super::TimeOffRequest;

#[derive(Debug)]
pub struct Agent {
    pub api_key: String,
    pub subdomain: String,
}

impl Agent {
    pub fn new(api_key: &str, subdomain: &str) -> Self {
        Self {
            api_key: api_key.into(),
            subdomain: subdomain.into(),
        }
    }

    fn api_prefix(&self) -> String {
        format!(
            "https://api.bamboohr.com/api/gateway.php/{}/v1",
            self.subdomain
        )
    }

    pub async fn get_time_off_requests(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<TimeOffRequest>, Box<(dyn std::error::Error + Send + Sync)>> {
        let resource = format!(
            "/time_off/requests/?start={}&end={}",
            start_date.format("%Y-%m-%d"),
            end_date.format("%Y-%m-%d"),
        );
        dbg!(&resource);

        let url = format!("{}{}", self.api_prefix(), resource);
        dbg!(&url);

        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .header("Accept", "application/json")
            .basic_auth(&self.api_key, Some("x"))
            .send()
            .await?;
        let body = resp.text().await?;

        let requests: Vec<TimeOffRequest> = serde_json::from_str(&body)?;
        Ok(requests)
    }

    pub async fn mock_time_off_requests(
        &self,
    ) -> Result<Vec<TimeOffRequest>, Box<(dyn std::error::Error + Send + Sync)>> {
        let body = tokio::fs::read_to_string("data/sample-dynamodb-event.json").await?;
        let requests: Vec<TimeOffRequest> = serde_json::from_str(&body)?;
        Ok(requests)
    }
}
