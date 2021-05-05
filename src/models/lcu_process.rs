use tokio::process::Command;

use crate::errors::LcuDriverError;
use crate::Result;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LcuProcess {
    output: String,
}

#[cfg(windows)]
impl LcuProcess {
    #[cfg(windows)]
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

        if output.contains("--install-directory=") {
            Ok(Self { output })
        } else {
            Err(LcuDriverError::FailedToFindLeagueProcess)
        }
    }

    #[cfg(not(windows))]
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

        Ok(Self { output })
    }

    pub fn get_argument_value(&self, argument: &str) -> Option<&str> {
        self.output
            .split(" --")
            .find(|l| l.starts_with(argument))
            .map(|l| l.trim_start_matches(argument))
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::LcuDriverError;
    use crate::models::lcu_process::LcuProcess;

    #[tokio::test]
    async fn test_spawn_fail() {
        let process = LcuProcess::locate().await;

        assert_eq!(process, Err(LcuDriverError::FailedToFindLeagueProcess))
    }

    #[test]
    fn get_argument_value_is_none() {
        let league_process = LcuProcess {
            output: "".to_owned(),
        };

        assert_eq!(
            league_process.get_argument_value("--install-directory="),
            None
        );
    }

    #[test]
    fn get_argument_value_is_some() {
        let league_process = LcuProcess {
            output: r#""C:/Riot Games/League of Legends/LeagueClientUx.exe" --no-rads --disable-self-update --region=EUW --locale=en_GB --respawn-command=LeagueClient.exe --no-proxy-server --install-directory=C:\Riot Games\League of Legends"#.to_owned()
        };

        assert_eq!(
            league_process.get_argument_value("install-directory="),
            Some("C:\\Riot Games\\League of Legends")
        );
    }
}
