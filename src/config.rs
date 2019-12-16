use serde::Deserialize;

#[derive(Deserialize, Default, PartialEq, Debug)]
pub struct Config {
    pub(crate) log_group_name: Option<String>,
    pub(crate) log_stream_name: Option<String>,
    pub(crate) enable_debug_logging: Option<String>,
    pub(crate) service_name: Option<String>,
    pub(crate) service_type: Option<String>,
    pub(crate) agent_endpoit: Option<String>,
}

pub fn get() -> Config {
    envy::prefixed("AWS_EMF_").from_env().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::set_var;
    #[test]
    fn it_deserializes_from_env() {
        set_var("AWS_EMF_SERVICE_NAME", "test");
        assert_eq!(
            get(),
            Config {
                service_name: Some("test".into()),
                ..Config::default()
            }
        );
    }
}
