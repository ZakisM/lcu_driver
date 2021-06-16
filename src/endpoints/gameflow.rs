use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::endpoints::EndpointInfo;

const CHAMP_SELECT_URL: &str = "/lol-gameflow/v1";

#[allow(unused)]
pub enum GameFlowEndpoint {
    Session,
}

impl GameFlowEndpoint {
    pub fn info(&self) -> EndpointInfo {
        match self {
            GameFlowEndpoint::Session => EndpointInfo {
                url: format!("{}/session", CHAMP_SELECT_URL),
                method: Method::GET,
                headers: None,
                body: None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFlowSession {
    pub game_client: GameClient,
    pub game_data: GameData,
    pub game_dodge: GameDodge,
    pub map: Map,
    pub phase: GameFlowPhase,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameFlowPhase {
    None,
    Lobby,
    Matchmaking,
    CheckedIntoTournament,
    ReadyCheck,
    ChampSelect,
    GameStart,
    FailedToLaunch,
    InProgress,
    Reconnect,
    WaitingForStats,
    PreEndOfGame,
    EndOfGame,
    TerminatedInError,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameClient {
    pub observer_server_ip: String,
    pub observer_server_port: isize,
    pub running: bool,
    pub server_ip: String,
    pub server_port: isize,
    pub visible: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub game_id: isize,
    pub game_name: String,
    pub is_custom_game: bool,
    pub password: String,
    pub player_champion_selections: Vec<::serde_json::Value>,
    pub queue: Queue,
    pub spectators_allowed: bool,
    pub team_one: Vec<::serde_json::Value>,
    pub team_two: Vec<::serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Queue {
    pub allowable_premade_sizes: Vec<isize>,
    pub are_free_champions_allowed: bool,
    pub asset_mutator: String,
    pub category: String,
    pub champions_required_to_play: isize,
    pub description: String,
    pub detailed_description: String,
    pub game_mode: String,
    pub game_type_config: GameTypeConfig,
    pub id: isize,
    pub is_ranked: bool,
    pub is_team_builder_managed: bool,
    pub is_team_only: bool,
    pub last_toggled_off_time: isize,
    pub last_toggled_on_time: isize,
    pub map_id: isize,
    pub max_level: isize,
    pub max_summoner_level_for_first_win_of_the_day: isize,
    pub maximum_participant_list_size: isize,
    pub min_level: isize,
    pub minimum_participant_list_size: isize,
    pub name: String,
    pub num_players_per_team: isize,
    pub queue_availability: String,
    pub queue_rewards: QueueRewards,
    pub removal_from_game_allowed: bool,
    pub removal_from_game_delay_minutes: isize,
    pub short_name: String,
    pub show_position_selector: bool,
    pub spectator_enabled: bool,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameTypeConfig {
    pub advanced_learning_quests: bool,
    pub allow_trades: bool,
    pub ban_mode: String,
    pub ban_timer_duration: isize,
    pub battle_boost: bool,
    pub cross_team_champion_pool: bool,
    pub death_match: bool,
    pub do_not_remove: bool,
    pub duplicate_pick: bool,
    pub exclusive_pick: bool,
    pub id: isize,
    pub learning_quests: bool,
    pub main_pick_timer_duration: isize,
    pub max_allowable_bans: isize,
    pub name: String,
    pub onboard_coop_beginner: bool,
    pub pick_mode: String,
    pub post_pick_timer_duration: isize,
    pub reroll: bool,
    pub team_champion_pool: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueRewards {
    pub is_champion_points_enabled: bool,
    pub is_ip_enabled: bool,
    pub is_xp_enabled: bool,
    pub party_size_ip_rewards: Vec<::serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDodge {
    pub dodge_ids: Vec<::serde_json::Value>,
    pub phase: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub assets: Assets,
    pub categorized_content_bundles: CategorizedContentBundles,
    pub description: String,
    #[serde(default)]
    pub game_mode: GameMode,
    pub game_mode_name: String,
    pub game_mode_short_name: String,
    pub game_mutator: String,
    pub id: isize,
    #[serde(rename = "isRGM")]
    pub is_rgm: bool,
    pub map_string_id: String,
    pub name: String,
    pub per_position_disallowed_summoner_spells: PerPositionDisallowedSummonerSpells,
    pub per_position_required_summoner_spells: PerPositionRequiredSummonerSpells,
    pub platform_id: String,
    pub platform_name: String,
    pub properties: Properties,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GameMode {
    Classic,
    Aram,
    PracticeTool,
    NexusBlitz,
    #[serde(other)]
    Unknown,
}

impl std::default::Default for GameMode {
    fn default() -> Self {
        Self::Unknown
    }
}

impl GameMode {
    pub fn disallowed_summoner_spells(&self) -> Option<Vec<isize>> {
        match self {
            GameMode::Classic
            | GameMode::PracticeTool
            | GameMode::NexusBlitz
            | GameMode::Unknown => None,
            GameMode::Aram => Some(vec![11, 12]),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assets {
    #[serde(rename = "champ-select-background-sound")]
    pub champ_select_background_sound: String,
    #[serde(rename = "champ-select-flyout-background")]
    pub champ_select_flyout_background: String,
    #[serde(rename = "champ-select-planning-intro")]
    pub champ_select_planning_intro: Option<String>,
    #[serde(rename = "game-select-icon-active")]
    pub game_select_icon_active: String,
    #[serde(rename = "game-select-icon-active-video")]
    pub game_select_icon_active_video: String,
    #[serde(rename = "game-select-icon-default")]
    pub game_select_icon_default: String,
    #[serde(rename = "game-select-icon-disabled")]
    pub game_select_icon_disabled: String,
    #[serde(rename = "game-select-icon-hover")]
    pub game_select_icon_hover: String,
    #[serde(rename = "game-select-icon-intro-video")]
    pub game_select_icon_intro_video: String,
    #[serde(rename = "gameflow-background")]
    pub gameflow_background: String,
    #[serde(rename = "gameselect-button-hover-sound")]
    pub gameselect_button_hover_sound: String,
    #[serde(rename = "icon-defeat")]
    pub icon_defeat: String,
    #[serde(rename = "icon-defeat-video")]
    pub icon_defeat_video: String,
    #[serde(rename = "icon-empty")]
    pub icon_empty: String,
    #[serde(rename = "icon-hover")]
    pub icon_hover: String,
    #[serde(rename = "icon-leaver")]
    pub icon_leaver: String,
    #[serde(rename = "icon-victory")]
    pub icon_victory: String,
    #[serde(rename = "icon-victory-video")]
    pub icon_victory_video: String,
    #[serde(rename = "map-north")]
    pub map_north: Option<String>,
    #[serde(rename = "map-south")]
    pub map_south: Option<String>,
    #[serde(rename = "music-inqueue-loop-sound")]
    pub music_inqueue_loop_sound: String,
    #[serde(rename = "parties-background")]
    pub parties_background: String,
    #[serde(rename = "postgame-ambience-loop-sound")]
    pub postgame_ambience_loop_sound: String,
    #[serde(rename = "ready-check-background")]
    pub ready_check_background: String,
    #[serde(rename = "ready-check-background-sound")]
    pub ready_check_background_sound: String,
    #[serde(rename = "sfx-ambience-pregame-loop-sound")]
    pub sfx_ambience_pregame_loop_sound: String,
    #[serde(rename = "social-icon-leaver")]
    pub social_icon_leaver: String,
    #[serde(rename = "social-icon-victory")]
    pub social_icon_victory: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorizedContentBundles {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerPositionDisallowedSummonerSpells {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerPositionRequiredSummonerSpells {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    pub suppress_runes_masteries_perks: bool,
}
