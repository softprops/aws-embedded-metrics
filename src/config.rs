use serde::Deserialize;

#[derive(Deserialize, Default, PartialEq)]
pub struct Config {
    pub(crate) log_group_name: Option<String>,
    pub(crate) log_stream_name: Option<String>,
    pub(crate) enable_debug_logging: Option<String>,
    pub(crate) service_name: Option<String>,
    pub(crate) service_type: Option<String>,
    pub(crate) agent_endpoit: Option<String>,
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
