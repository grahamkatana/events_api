use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct Storage {
    client: Client,
    bucket: String,
    public_url: String,
}

impl Storage {
    pub async fn from_env() -> Self {
        let endpoint = std::env::var("MINIO_ENDPOINT").expect("MINIO_ENDPOINT must be set");
        let access_key = std::env::var("MINIO_ACCESS_KEY").expect("MINIO_ACCESS_KEY must be set");
        let secret_key = std::env::var("MINIO_SECRET_KEY").expect("MINIO_SECRET_KEY must be set");
        let bucket = std::env::var("MINIO_BUCKET").expect("MINIO_BUCKET must be set");
        let public_url = std::env::var("MINIO_PUBLIC_URL").expect("MINIO_PUBLIC_URL must be set");

        let credentials = Credentials::new(access_key, secret_key, None, None, "minio");

        let config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .region(Region::new("us-east-1"))
            // MinIO needs "path style" URLs (host/bucket/key) instead of
            // AWS's usual "virtual host style" (bucket.host/key) — real
            // S3 supports both, but MinIO only supports this one.
            .force_path_style(true)
            .build();

        let client = Client::from_conf(config);

        let storage = Storage {
            client,
            bucket,
            public_url,
        };
        storage.ensure_bucket_exists().await;
        storage
    }

    // Creates the bucket (and makes it publicly readable) the first
    // time the app starts — so you don't need to click through MinIO's
    // console manually before anything works.
    async fn ensure_bucket_exists(&self) {
        let exists = self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok();

        if exists {
            return;
        }

        let _ = self.client.create_bucket().bucket(&self.bucket).send().await;

        let policy = format!(
            r#"{{
                "Version": "2012-10-17",
                "Statement": [{{
                    "Effect": "Allow",
                    "Principal": "*",
                    "Action": ["s3:GetObject"],
                    "Resource": ["arn:aws:s3:::{bucket}/*"]
                }}]
            }}"#,
            bucket = self.bucket
        );

        let _ = self
            .client
            .put_bucket_policy()
            .bucket(&self.bucket)
            .policy(policy)
            .send()
            .await;
    }

    pub async fn upload(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<String, String> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("{}/{}/{}", self.public_url, self.bucket, key))
    }
}