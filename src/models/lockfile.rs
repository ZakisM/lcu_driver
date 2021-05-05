use std::path::PathBuf;

use crate::errors::LcuDriverError;
use crate::Result;

#[derive(Debug, Clone)]
pub struct Lockfile {
    pub path: PathBuf,
    pub port: isize,
    pub token: String,
    contents: String,
}

impl Lockfile {
    pub async fn load(path: PathBuf) -> Result<Self> {
        let contents = tokio::fs::read_to_string(&path).await?;
        let lockfile_items = contents.split(':').collect::<Vec<_>>();

        let port = lockfile_items
            .get(2)
            .ok_or(LcuDriverError::FailedToReadLockfileToken)?
            .parse()?;

        let decoded_token = lockfile_items
            .get(3)
            .ok_or(LcuDriverError::FailedToReadLockfileToken)?;

        let full_decoded = format!("riot:{}", decoded_token);
        let token = base64::encode(full_decoded);

        let path = path.to_path_buf();

        Ok(Self {
            path,
            port,
            token,
            contents,
        })
    }

    pub async fn exists(&self) -> bool {
        self.path.exists()
    }

    pub async fn contents_changed(&self) -> bool {
        if let Ok(contents) = tokio::fs::read_to_string(&self.path).await {
            self.contents != contents
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::models::lockfile::Lockfile;

    #[tokio::test]
    async fn test_token() {
        let lockfile = Lockfile::load(Path::new("./test_data/lockfile").to_path_buf())
            .await
            .expect("Failed to load test file");

        assert_eq!(lockfile.port, 50261);
        assert_eq!(lockfile.token, "cmlvdDpxU3h2TGFNSGdxMTdteFVLYUZmU2Rn");
    }
}
