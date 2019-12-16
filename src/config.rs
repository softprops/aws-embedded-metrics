use serde::Deserialize;

#[derive(Deserialize, Default, PartialEq)]
pub struct Config {
    log_group_name: Option<String>,
    log_stream_name: Option<String>,
    enable_debug_logging: Option<String>,
    service_name: Option<String>,
    service_type: Option<String>,
    agent_endpoit: Option<String>,
}

pub fn get() -> Config {
    envy::prefixed("AWS_EMF").from_env().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_deserializes_from_env() {
        assert_eq!(2 + 2, 4);
    }
}
