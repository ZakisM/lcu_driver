use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::endpoints::EndpointInfo;

const SUMMONER_URL: &str = "/lol-summoner/v1";

#[allow(unused)]
pub enum SummonerEndpoint {
    Current,
}

impl SummonerEndpoint {
    pub fn info(&self) -> EndpointInfo {
        match self {
            SummonerEndpoint::Current => EndpointInfo {
                url: format!("{}/current-summoner", SUMMONER_URL),
                method: Method::GET,
                headers: None,
                body: None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summoner {
    pub account_id: isize,
    pub display_name: String,
    pub internal_name: String,
    pub name_change_flag: bool,
    pub percent_complete_for_next_level: isize,
    pub profile_icon_id: isize,
    pub puuid: String,
    pub reroll_points: RerollPoints,
    pub summoner_id: isize,
    pub summoner_level: isize,
    pub unnamed: bool,
    pub xp_since_last_level: isize,
    pub xp_until_next_level: isize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RerollPoints {
    pub current_points: isize,
    pub max_rolls: isize,
    pub number_of_rolls: isize,
    pub points_cost_to_roll: isize,
    pub points_to_reroll: isize,
}
