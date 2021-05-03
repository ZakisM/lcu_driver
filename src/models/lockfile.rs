use std::path::PathBuf;

use crate::errors::LcuDriverError;
use crate::Result;

#[derive(Debug, Clone)]
pub struct Lockfile {
    pub path: PathBuf,
    pub port: isize,
    pub token: String,
}

impl Lockfile {
    pub async fn load(path: PathBuf) -> Result<Self> {
        let data = tokio::fs::read_to_string(&path).await?;
        let data_items = data.split(':').collect::<Vec<_>>();

        let port = data_items
            .get(2)
            .ok_or(LcuDriverError::FailedToReadLockfileToken)?
            .parse()?;

        let decoded_token = data_items
            .get(3)
            .ok_or(LcuDriverError::FailedToReadLockfileToken)?;
        let full_decoded = format!("riot:{}", decoded_token);
        let token = base64::encode(full_decoded);

        let path = path.to_path_buf();

        Ok(Self { path, port, token })
    }

    pub async fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::models::lockfile::Lockfile;

    #[tokio::test]
    async fn test_token() {
        let lockfile = Lockfile::load(Path::new("./test_data/lockfile"))
            .await
            .expect("Failed to load test file");

        assert_eq!(lockfile.port, 50261);
        assert_eq!(lockfile.token, "cmlvdDpxU3h2TGFNSGdxMTdteFVLYUZmU2Rn");
    }
}
