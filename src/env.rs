use crate::{config::Config, log::MetricContext};
use serde::Deserialize;
use std::env::var;

pub trait EnvironmentProvider {
    fn get(&mut self) -> &dyn Env;
}

pub struct Detector {
    potentials: Vec<Box<dyn Env>>,
    fallback: Box<dyn Env>,
}

impl Default for Detector {
    fn default() -> Detector {
        Detector {
            potentials: vec![Box::new(Lambda), Box::new(EC2::new())],
            fallback: Box::new(Fallback(crate::config::get())),
        }
    }
}

impl EnvironmentProvider for Detector {
    fn get(&mut self) -> &dyn Env {
        for env in &self.potentials {
            if env.probe() {
                return env.as_ref();
            }
        }
        self.fallback.as_ref()
    }
}

pub trait Env {
    fn probe(&self) -> bool;
    fn name(&self) -> String;
    fn env_type(&self) -> String;
    fn log_group_name(&self) -> String;
    fn configure(
        &self,
        context: &mut MetricContext,
    ) -> ();
}

pub struct Fallback(Config);

impl Env for Fallback {
    fn probe(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        self.0
            .service_name
            .clone()
            .unwrap_or_else(|| "Unknown".into())
    }

    fn env_type(&self) -> String {
        self.0
            .service_type
            .clone()
            .unwrap_or_else(|| "Unknown".into())
    }

    fn log_group_name(&self) -> String {
        self.0
            .log_group_name
            .clone()
            .unwrap_or_else(|| format!("{}-metrics", self.name()))
    }

    fn configure(
        &self,
        _: &mut MetricContext,
    ) {
    }
}

pub struct Lambda;

impl Env for Lambda {
    fn probe(&self) -> bool {
        var("AWS_LAMBDA_FUNCTION_NAME").is_ok()
    }

    fn name(&self) -> String {
        var("AWS_LAMBDA_FUNCTION_NAME").unwrap_or_else(|_| "Unknown".into())
    }

    fn env_type(&self) -> String {
        "AWS::Lambda::Function".into()
    }

    fn log_group_name(&self) -> String {
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

pub struct EC2 {
    config: Config,
    metadata: Option<EC2MetadataResponse>,
}

impl EC2 {
    fn new() -> Self {
        Self {
            config: crate::config::get(),
            metadata: None,
        }
    }
}

impl Env for EC2 {
    fn probe(&self) -> bool {
        false
    }

    fn name(&self) -> String {
        self.config
            .service_name
            .clone()
            .unwrap_or_else(|| "Unknown".into())
    }

    fn env_type(&self) -> String {
        if self.metadata.is_some() {
            "AWS::EC2::Instance".into()
        } else {
            "Unknown".into()
        }
    }

    fn log_group_name(&self) -> String {
        self.config
            .service_name
            .clone()
            .unwrap_or_else(|| format!("{}-metrics", self.name()))
    }

    fn configure(
        &self,
        context: &mut MetricContext,
    ) {
        if let Some(metadata) = &self.metadata {
            context.set_property("imageId", metadata.image_id.as_str());
            context.set_property("instanceId", metadata.instance_id.as_str());
            context.set_property("instanceType", metadata.instance_type.as_str());
            context.set_property("privateIP", metadata.private_ip.as_str());
            context.set_property("availabilityZone", metadata.availability_zone.as_str());
        }
    }
}
