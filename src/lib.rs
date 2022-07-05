pub mod alertdb;
pub mod bamboo;
pub(crate) mod format;

pub fn get_required_env_var(
    var_name: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::env;
    env::var(var_name)
        .map_err(|_| format!("error: required env var {} is not set", var_name).into())
}
