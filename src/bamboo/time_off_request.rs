use chrono::NaiveDate;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};

use crate::format::my_date_format;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeOffRequest {
    #[serde(deserialize_with = "i64_from_string")]
    pub id: i64,
    pub name: String,
    #[serde(alias = "start", with = "my_date_format")]
    pub start_date: NaiveDate,
    #[serde(alias = "end", with = "my_date_format")]
    pub end_date: NaiveDate,
    #[serde(alias = "created", with = "my_date_format")]
    pub created_date: NaiveDate,
}

impl TimeOffRequest {
    pub fn format_slack_msg(&self) -> String {
        format!(
            "{} has requested time off for {} to {}.",
            self.name, self.start_date, self.end_date
        )
    }
}

pub fn i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<i64>().map_err(de::Error::custom)?,
        Value::Number(num) => num
            .as_i64()
            .ok_or_else(|| de::Error::custom("Invalid number"))?,
        _ => return Err(de::Error::custom("wrong type")),
    })
}
