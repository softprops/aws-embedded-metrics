use serde::Deserialize;
use std::env::vars;

#[derive(Deserialize, Default, PartialEq, Debug)]
pub struct Config {
    pub(crate) log_group_name: Option<String>,
    pub(crate) log_stream_name: Option<String>,
    pub(crate) enable_debug_logging: Option<String>,
    pub(crate) service_name: Option<String>,
    pub(crate) service_type: Option<String>,
    pub(crate) agent_endpoint: Option<String>,
}

pub fn get() -> Config {
    from_vars(vars())
}

fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> Config {
    envy::prefixed("AWS_EMF_")
        .from_iter(vars)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::set_var;

    #[test]
    fn it_defaults_when_no_data_is_provided() {
        assert_eq!(from_vars(Vec::new()), Config::default())
    }

    #[test]
    fn it_deserializes_from_env() {
        for (key, value) in &[
            ("LOG_GROUP_NAME", "a"),
            ("LOG_STREAM_NAME", "b"),
            ("ENABLE_DEBUG_LOGGING", "c"),
            ("SERVICE_NAME", "d"),
            ("SERVICE_TYPE", "e"),
            ("AGENT_ENDPOINT", "f"),
        ] {
            set_var(format!("AWS_EMF_{}", key), value);
        }
        assert_eq!(
            get(),
            Config {
                log_group_name: Some("a".into()),
                log_stream_name: Some("b".into()),
                enable_debug_logging: Some("c".into()),
                service_name: Some("d".into()),
                service_type: Some("e".into()),
                agent_endpoint: Some("f".into())
            }
        );
    }
}
