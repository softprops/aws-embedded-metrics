use crate::{config::Config, log::MetricContext};
use serde::Deserialize;
use std::{
    borrow::Cow,
    env::var,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    time::Duration,
};

pub(crate) trait EnvironmentProvider {
    fn get(&mut self) -> Box<dyn Env>;
}

pub(crate) struct Detector;

impl EnvironmentProvider for Detector {
    fn get(&mut self) -> Box<dyn Env> {
        let potentials: Vec<Box<dyn Env + 'static>> = vec![Box::new(Lambda), Box::new(EC2::new())];
        for mut env in potentials.into_iter() {
            if env.probe() {
                return env;
            }
        }
        Box::new(Vars(crate::config::get()))
    }
}

pub(crate) trait Env {
    fn probe(&mut self) -> bool;
    fn name(&self) -> Cow<'_, str>;
    fn env_type(&self) -> Cow<'_, str>;
    fn log_group_name(&self) -> Cow<'_, str>;
    fn configure(
        &self,
        context: &mut MetricContext,
    ) -> ();
}

pub(crate) struct Vars(Config);

impl Env for Vars {
    fn probe(&mut self) -> bool {
        true
    }

    fn name(&self) -> Cow<'_, str> {
        self.0
            .service_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| "Unknown")
            .into()
    }

    fn env_type(&self) -> Cow<'_, str> {
        self.0
            .service_type
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| "Unknown")
            .into()
    }

    fn log_group_name(&self) -> Cow<'_, str> {
        self.0
            .log_group_name
            .clone()
            .unwrap_or_else(|| format!("{}-metrics", self.name()))
            .into()
    }

    fn configure(
        &self,
        _: &mut MetricContext,
    ) {
    }
}

pub(crate) struct Lambda;

impl Env for Lambda {
    fn probe(&mut self) -> bool {
        var("AWS_LAMBDA_FUNCTION_NAME").is_ok()
    }

    fn name(&self) -> Cow<'_, str> {
        var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "Unknown".into())
            .into()
    }

    fn env_type(&self) -> Cow<'_, str> {
        "AWS::Lambda::Function".into()
    }

    fn log_group_name(&self) -> Cow<'_, str> {
        self.name()
    }

    fn configure(
        &self,
        context: &mut MetricContext,
    ) {
        if let Ok(value) = var("AWS_EXECUTION_ENV") {
            context.set_property("executionEnvironment", value);
        }
        if let Ok(value) = var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE") {
            context.set_property("memorySize", value);
        }
        if let Ok(value) = var("AWS_LAMBDA_FUNCTION_VERSION") {
            context.set_property("functionVersion", value);
        }
        if let Ok(value) = var("AWS_LAMBDA_LOG_STREAM_NAME") {
            context.set_property("logStreamId", value);
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EC2MetadataResponse {
    image_id: String,
    availability_zone: String,
    private_ip: String,
    instance_id: String,
    instance_type: String,
}

enum EC2Error {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

pub(crate) struct EC2 {
    config: Config,
    metadata: Option<Result<EC2MetadataResponse, EC2Error>>,
}

impl EC2 {
    fn new() -> Self {
        Self {
            config: crate::config::get(),
            metadata: None,
        }
    }

    /// fetch ec2 instance metadata from well known http endpont
    fn fetch(&self) -> Result<EC2MetadataResponse, EC2Error> {
        // https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instancedata-data-retrieval.html
        let mut conn = TcpStream::connect_timeout(
            &([169, 254, 169, 254], 80).into(),
            Duration::from_millis(50),
        )
        .map_err(EC2Error::Io)?;
        conn.set_read_timeout(Some(Duration::from_millis(50)))
            .map_err(EC2Error::Io)?;

        conn.write_all(
            b"GET /latest/dynamic/instance-identity/document HTTP/1.1\r\nHost: 169.254.169.254\r\n\r\n",
        )
        .map_err(EC2Error::Io)?;

        let response = BufReader::new(conn).lines().filter_map(Result::ok).skip(9);
        serde_json::from_str(&response.collect::<Vec<_>>().join("")).map_err(EC2Error::Parse)
    }
}

impl Env for EC2 {
    fn probe(&mut self) -> bool {
        if self.metadata.is_some() {
            return self.metadata.as_ref().iter().any(|m| m.is_ok());
        }
        self.metadata = Some(self.fetch());
        self.probe()
    }

    fn name(&self) -> Cow<'_, str> {
        self.config
            .service_name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| "Unknown")
            .into()
    }

    fn env_type(&self) -> Cow<'_, str> {
        if self.metadata.is_some() {
            "AWS::EC2::Instance".into()
        } else {
            "Unknown".into()
        }
    }

    fn log_group_name(&self) -> Cow<'_, str> {
        self.config
            .service_name
            .as_ref()
            .map(|s| s.into())
            .unwrap_or_else(|| format!("{}-metrics", self.name()).into())
    }

    fn configure(
        &self,
        context: &mut MetricContext,
    ) {
        if let Some(Ok(metadata)) = &self.metadata {
            context.set_property("imageId", metadata.image_id.as_str());
            context.set_property("instanceId", metadata.instance_id.as_str());
            context.set_property("instanceType", metadata.instance_type.as_str());
            context.set_property("privateIP", metadata.private_ip.as_str());
            context.set_property("availabilityZone", metadata.availability_zone.as_str());
        }
    }
}
