pub mod structures;

use self::structures::*;
use reqwest::{Client, StatusCode};
use {path_segment_encode, query_encode, EurekaError};

#[derive(Debug)]
pub struct EurekaRestClient {
    client: Client,
    base_url: String,
}

impl EurekaRestClient {
    pub fn new(base_url: String) -> Self {
        EurekaRestClient {
            client: Client::new(),
            base_url,
        }
    }

    /// Register new application instance
    pub fn register(&self, app_id: &str, data: &Instance) -> Result<(), EurekaError> {
        let url = format!("{}/apps/{}", self.base_url, path_segment_encode(app_id));
        debug!("Sending register request to {}", url);
        let resp = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .json(&Register { instance: data })
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(resp) => match resp.status() {
                StatusCode::NO_CONTENT => Ok(()),
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// De-register application instance
    pub fn deregister(&self, app_id: &str, instance_id: &str) -> Result<(), EurekaError> {
        let url = format!(
            "{}/apps/{}/{}",
            self.base_url,
            path_segment_encode(app_id),
            path_segment_encode(instance_id)
        );
        debug!("Sending deregister request to {}", url);
        let resp = self.client.delete(&url).send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(resp) => match resp.status() {
                StatusCode::OK => Ok(()),
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Send application instance heartbeat
    pub fn send_heartbeat(&self, app_id: &str, instance_id: &str) -> Result<(), EurekaError> {
        let url = format!(
            "{}/apps/{}/{}",
            self.base_url,
            path_segment_encode(app_id),
            path_segment_encode(instance_id)
        );
        debug!("Sending heartbeat request to {}", url);
        let resp = self
            .client
            .put(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(resp) => match resp.status() {
                StatusCode::OK => Ok(()),
                StatusCode::NOT_FOUND => Err(EurekaError::UnexpectedState(
                    "Instance does not exist".into(),
                )),
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Query for all instances
    pub fn get_all_instances(&self) -> Result<Vec<Instance>, EurekaError> {
        let url = format!("{}/apps", self.base_url);
        debug!("Sending get all instances request to {}", url);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(mut resp) => match resp.status() {
                StatusCode::OK => {
                    let apps: AllApplications = resp
                        .json()
                        .map_err(|e| EurekaError::ParseError(e.to_string()))?;
                    Ok(apps
                        .applications
                        .application
                        .into_iter()
                        .flat_map(|a| a.instance.into_iter())
                        .collect())
                }
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Query for all `app_id` instances
    pub fn get_instances_by_app(&self, app_id: &str) -> Result<Vec<Instance>, EurekaError> {
        let url = format!("{}/apps/{}", self.base_url, path_segment_encode(app_id));
        debug!("Sending get instances by app request to {}", url);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(mut resp) => match resp.status() {
                StatusCode::OK => {
                    let apps: ApplicationWrapper = resp
                        .json()
                        .map_err(|e| EurekaError::ParseError(e.to_string()))?;
                    Ok(apps.application.instance)
                }
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Query for a specific `app_id/instance_id`
    pub fn get_instance_by_app_and_instance(
        &self,
        app_id: &str,
        instance_id: &str,
    ) -> Result<Instance, EurekaError> {
        let url = format!(
            "{}/apps/{}/{}",
            self.base_url,
            path_segment_encode(app_id),
            path_segment_encode(instance_id)
        );
        debug!(
            "Sending get instance by app and instance request to {}",
            url
        );
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(mut resp) => match resp.status() {
                StatusCode::OK => {
                    let apps: InstanceWrapper = resp
                        .json()
                        .map_err(|e| EurekaError::ParseError(e.to_string()))?;
                    Ok(apps.instance)
                }
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Update instance status
    pub fn update_status(
        &self,
        app_id: &str,
        instance_id: &str,
        new_status: StatusType,
    ) -> Result<(), EurekaError> {
        let url = format!(
            "{}/apps/{}/{}/status?value={}",
            self.base_url,
            path_segment_encode(app_id),
            path_segment_encode(instance_id),
            new_status
        );
        debug!("Sending update status request to {}", url);
        let resp = self
            .client
            .put(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(resp) => match resp.status() {
                StatusCode::OK => Ok(()),
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Update metadata
    pub fn update_metadata(
        &self,
        app_id: &str,
        instance_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), EurekaError> {
        let url = format!(
            "{}/apps/{}/{}/metadata?{}={}",
            self.base_url,
            path_segment_encode(app_id),
            path_segment_encode(instance_id),
            query_encode(key),
            query_encode(value)
        );
        debug!("Sending update metadata request to {}", url);
        let resp = self
            .client
            .put(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(resp) => match resp.status() {
                StatusCode::OK => Ok(()),
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Query for all instances under a particular `vip_address`
    pub fn get_instances_by_vip_address(
        &self,
        vip_address: &str,
    ) -> Result<Vec<Instance>, EurekaError> {
        let url = format!(
            "{}/vips/{}",
            self.base_url,
            path_segment_encode(vip_address)
        );
        debug!("Sending get instances by vip address request to {}", url);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(mut resp) => match resp.status() {
                StatusCode::OK => {
                    let apps: AllApplications = resp
                        .json()
                        .map_err(|e| EurekaError::ParseError(e.to_string()))?;
                    Ok(apps
                        .applications
                        .application
                        .into_iter()
                        .flat_map(|a| a.instance.into_iter())
                        .collect())
                }
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }

    /// Query for all instances under a particular `svip_address`
    pub fn get_instances_by_svip_address(
        &self,
        svip_address: &str,
    ) -> Result<Vec<Instance>, EurekaError> {
        let url = format!(
            "{}/svips/{}",
            self.base_url,
            path_segment_encode(svip_address)
        );
        debug!("Sending get instances by svip address request to {}", url);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send();
        match resp {
            Err(e) => Err(EurekaError::Network(e)),
            Ok(mut resp) => match resp.status() {
                StatusCode::OK => {
                    let apps: AllApplications = resp
                        .json()
                        .map_err(|e| EurekaError::ParseError(e.to_string()))?;
                    Ok(apps
                        .applications
                        .application
                        .into_iter()
                        .flat_map(|a| a.instance.into_iter())
                        .collect())
                }
                _ => Err(EurekaError::Request(resp.status())),
            },
        }
    }
}
