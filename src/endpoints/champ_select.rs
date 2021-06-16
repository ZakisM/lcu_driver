use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::endpoints::EndpointInfo;

const CHAMP_SELECT_URL: &str = "/lol-champ-select/v1";

#[allow(unused)]
pub enum ChampSelectEndpoint<'a> {
    Session,
    SessionMySelection(&'a str),
}

impl<'a> ChampSelectEndpoint<'a> {
    pub fn info(&self) -> EndpointInfo {
        match self {
            ChampSelectEndpoint::Session => EndpointInfo {
                url: format!("{}/session", CHAMP_SELECT_URL),
                method: Method::GET,
                headers: None,
                body: None,
            },
            ChampSelectEndpoint::SessionMySelection(body) => EndpointInfo {
                url: format!("{}/session/my-selection", CHAMP_SELECT_URL),
                method: Method::PATCH,
                headers: None,
                body: Some(body.to_string()),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    pub actions: Option<Vec<Vec<Action>>>,
    pub allow_battle_boost: Option<bool>,
    pub allow_duplicate_picks: bool,
    pub allow_locked_events: bool,
    pub allow_rerolling: bool,
    pub allow_skin_selection: bool,
    pub bans: Bans,
    pub bench_champion_ids: Vec<isize>,
    pub bench_enabled: bool,
    pub boostable_skin_count: isize,
    pub chat_details: ChatDetails,
    pub counter: isize,
    pub entitled_feature_state: EntitledFeatureState,
    pub game_id: isize,
    pub has_simultaneous_bans: bool,
    pub has_simultaneous_picks: bool,
    pub is_custom_game: bool,
    pub is_spectating: bool,
    pub local_player_cell_id: isize,
    pub locked_event_index: isize,
    pub my_team: Vec<PlayerSelection>,
    pub rerolls_remaining: isize,
    pub skip_champion_select: bool,
    pub their_team: Vec<PlayerSelection>,
    pub timer: Timer,
    pub trades: Vec<TradeContract>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub actor_cell_id: isize,
    pub champion_id: isize,
    pub completed: bool,
    pub id: isize,
    pub is_ally_action: bool,
    pub is_in_progress: bool,
    pub pick_turn: Option<isize>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bans {
    pub my_team_bans: Vec<BannedChampions>,
    pub num_bans: isize,
    pub their_team_bans: Vec<BannedChampions>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BannedChampions {
    my_team_bans: Vec<isize>,
    num_bans: isize,
    their_team_bans: Vec<isize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDetails {
    pub chat_room_name: Option<String>,
    pub chat_room_password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitledFeatureState {
    pub additional_rerolls: isize,
    pub unlocked_skin_ids: Vec<isize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSelection {
    pub assigned_position: String,
    pub cell_id: isize,
    pub champion_id: isize,
    pub champion_pick_intent: isize,
    pub entitled_feature_type: String,
    pub selected_skin_id: isize,
    pub spell1_id: isize,
    pub spell2_id: isize,
    pub summoner_id: isize,
    pub team: isize,
    pub ward_skin_id: isize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub adjusted_time_left_in_phase: isize,
    pub internal_now_in_epoch_ms: isize,
    pub is_infinite: bool,
    pub phase: String,
    pub total_time_in_phase: isize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeContract {
    cell_id: isize,
    id: isize,
    state: TradeContractState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeContractState {
    Available,
    Busy,
    Invalid,
    Received,
    Sent,
    Declined,
    Cancelled,
    Accepted,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MySelection {
    pub selected_skin_id: isize,
    pub spell_1_id: isize,
    pub spell_2_id: isize,
    pub ward_skin_id: isize,
}
