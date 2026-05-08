use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

use crate::config::Config;

#[derive(Clone)]
pub struct SpacesStorage {
    client: Client,
    bucket: String,
    endpoint: String,
}

impl SpacesStorage {
    pub async fn new(config: &Config) -> anyhow::Result<Self> {
        let credentials = Credentials::new(
            &config.spaces_key,
            &config.spaces_secret,
            None,
            None,
            "typeset",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .region(Region::new(config.spaces_region.clone()))
            .endpoint_url(&config.spaces_endpoint)
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Ok(SpacesStorage {
            client,
            bucket: config.spaces_bucket.clone(),
            endpoint: config.spaces_endpoint.clone(),
        })
    }

    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> anyhow::Result<String> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload to Spaces: {e}"))?;

        let url = format!(
            "{}/{}/{}",
            self.endpoint, self.bucket, key
        );
        Ok(url)
    }
}
