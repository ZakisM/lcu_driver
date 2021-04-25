use tokio::process::Command;

use crate::models::errors::LcuHelperError;
use crate::Result;

#[derive(Debug)]
pub struct LeagueClientUxProcess {
    output: String,
}

#[cfg(windows)]
impl LeagueClientUxProcess {
    pub async fn spawn() -> Result<Self> {
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

        let all_output = String::from_utf8(command.stdout.to_vec())?;

        let output_start = all_output
            .find("\r\r\n\"")
            .ok_or(LcuHelperError::FailedToFindLeagueProcess)?;

        let output = (&all_output[output_start..]).trim().to_owned();

        if output.contains("--install-directory=") {
            Ok(Self { output })
        } else {
            Err(LcuHelperError::FailedToFindLeagueProcess)
        }
    }

    pub fn get_argument_value(&self, argument: &str) -> Option<&str> {
        self.output
            .split("\" \"")
            .find(|l| l.starts_with(argument))
            .and_then(|l| l.strip_prefix(argument))
            .and_then(|l| l.strip_suffix("\""))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::league_ux_process::LeagueClientUxProcess;

    #[test]
    fn get_argument_value_is_none() {
        let league_process = LeagueClientUxProcess {
            output: "".to_owned()
        };

        assert_eq!(league_process.get_argument_value("--install-directory"), None);
    }

    #[test]
    fn get_argument_value_is_some() {
        let league_process = LeagueClientUxProcess {
            output: r#""C:/Riot Games/League of Legends/LeagueClientUx.exe" "--no-rads" "--disable-self-update" "--region=EUW" "--locale=en_GB" "--respawn-command=LeagueClient.exe" "--no-proxy-server" "--install-directory=C:\Riot Games\League of Legends""#.to_owned()
        };

        assert_eq!(league_process.get_argument_value("--install-directory="), Some("C:\\Riot Games\\League of Legends"));
    }
}
