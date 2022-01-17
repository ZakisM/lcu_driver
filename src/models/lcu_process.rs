use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::errors::LcuDriverError;
use crate::Result;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LcuProcess {
    output: String,
    install_directory: PathBuf,
}

impl LcuProcess {
    fn new(output: String) -> Result<Self> {
        let install_directory = PathBuf::from(
            Self::argument_value(&output, "install-directory=")
                .ok_or(LcuDriverError::FailedToFindLeagueProcess)?,
        );

        Ok(Self {
            output,
            install_directory,
        })
    }

    // currently only detects league installed through lutris
    #[cfg(target_os = "linux")]
    async fn new_lutris(output: String) -> Result<Self> {
        let mut process = Self::new(output)?;

        let lutris_command = Command::new("lutris").arg("-l").output().await?;

        if !lutris_command.status.success() {
            return Err(LcuDriverError::FailedToFindLutrisPrefix);
        }

        let lutris_output = String::from_utf8(lutris_command.stdout.to_vec())?;

        let lutris_prefix = lutris_output
            .split('|')
            .last()
            .map(|s| s.trim())
            .ok_or(LcuDriverError::FailedToFindLutrisPrefix)?;

        let wine_install_dir = process
            .install_directory
            .to_str()
            .map(|s| s.replace("\\", "/"))
            .map(|s| s.replace("C:/", "drive_c/"))
            .ok_or(LcuDriverError::FailedToFindLutrisPrefix)?;

        let lutris_install_directory = PathBuf::from(lutris_prefix).join(wine_install_dir);

        if lutris_install_directory.exists() {
            process.install_directory = lutris_install_directory;

            Ok(process)
        } else {
            Err(LcuDriverError::FailedToFindLutrisPrefix)
        }
    }

    #[cfg(target_os = "windows")]
    pub async fn locate() -> Result<Self> {
        let command = Command::new("cmd")
            .arg("/c")
            .arg("WMIC")
            .arg("PROCESS")
            .arg("WHERE")
            .arg("name='LeagueClientUx.exe'")
            .arg("GET")
            .arg("commandline")
            .output()
            .await?;

        if !command.status.success() {
            return Err(LcuDriverError::FailedToFindLeagueProcess);
        }

        let all_output = String::from_utf8(command.stdout.to_vec())?;

        let output_start = all_output
            .find("\r\r\n\"")
            .ok_or(LcuDriverError::FailedToFindLeagueProcess)?;

        let output_untrimmed = (&all_output[output_start..]).trim().to_owned();

        let output = output_untrimmed
            .split(' ')
            .map(|s| s.trim_start_matches('\"'))
            .map(|s| s.trim_end_matches('\"'))
            .collect::<Vec<_>>()
            .join(" ");

        let process = Self::new(output)?;

        Ok(process)
    }

    #[cfg(target_os = "macos")]
    pub async fn locate() -> Result<Self> {
        let command = Command::new("ps")
            .arg("x")
            .arg("-o")
            .arg("args")
            .output()
            .await?;

        if !command.status.success() {
            return Err(LcuDriverError::FailedToFindLeagueProcess);
        }

        let all_output = String::from_utf8(command.stdout.to_vec())?;

        let output = all_output
            .lines()
            .filter(|l| l.contains("LeagueClientUx"))
            .find(|l| l.contains("--install-directory="))
            .ok_or(LcuDriverError::FailedToFindLeagueProcess)?
            .to_owned();

        #[cfg(not(target_os = "linux"))]
        let process =
            Self::new(output).expect("Should never fail to find league install directory");

        #[cfg(target_os = "linux")]
        let process = Self::new_lutris(output)
            .await
            .expect("Should never fail to find league install directory");

        Ok(process)
    }

    pub fn install_directory(&self) -> &Path {
        &self.install_directory
    }

    fn argument_value<'a>(data: &'a str, argument: &str) -> Option<&'a str> {
        data.split(" --")
            .find(|l| l.starts_with(argument))
            .map(|l| l.trim_start_matches(argument))
    }

    pub fn get_argument_value(&self, argument: &str) -> Option<&str> {
        Self::argument_value(&self.output, argument)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_fail() {
        let process = LcuProcess::locate().await;

        assert_eq!(process, Err(LcuDriverError::FailedToFindLeagueProcess))
    }

    #[test]
    fn get_argument_value_is_none() {
        let league_process = LcuProcess {
            output: "".to_owned(),
            install_directory: PathBuf::new(),
        };

        assert_eq!(
            league_process.get_argument_value("--install-directory="),
            None
        );
    }

    #[test]
    fn get_argument_value_is_some() {
        let league_process = LcuProcess {
            output: r#""C:/Riot Games/League of Legends/LeagueClientUx.exe" --no-rads --disable-self-update --region=EUW --locale=en_GB --respawn-command=LeagueClient.exe --no-proxy-server --install-directory=C:\Riot Games\League of Legends"#.to_owned(),
            install_directory: PathBuf::new(),
        };

        assert_eq!(
            league_process.install_directory(),
            Path::new("C:\\Riot Games\\League of Legends")
        );
    }
}
