const SUMMONER_URL: &str = "/lol-summoner/v1";

#[allow(unused)]
pub enum SummonerEndpoint {
    Current,
}

impl SummonerEndpoint {
    pub fn url(&self) -> String {
        match self {
            SummonerEndpoint::Current => format!("{}/current-summoner", SUMMONER_URL),
        }
    }
}

// GET https://127.0.0.1:52995/lol-champ-select/v1/session
// actorCellId championId = current selected champ
// {
// "actions": [
// [
// {
// "actorCellId": 0,
// "championId": 103,
// "completed": false,
// "id": 1,
// "isAllyAction": true,
// "isInProgress": true,
// "pickTurn": 1,
// "type": "pick"
// }
// ]
// ],
// "allowBattleBoost": false,
// "allowDuplicatePicks": false,
// "allowLockedEvents": false,
// "allowRerolling": false,
// "allowSkinSelection": true,
// "bans": {
// "myTeamBans": [],
// "numBans": 0,
// "theirTeamBans": []
// },
// "benchChampionIds": [],
// "benchEnabled": false,
// "boostableSkinCount": 1,
// "chatDetails": {
// "chatRoomName": "c1~b0cb1b12470760f6d37944146b6bb8ec36411acf@sec.pvp.net",
// "chatRoomPassword": "cJv18rWCY7BhgqhY"
// },
// "counter": -1,
// "entitledFeatureState": {
// "additionalRerolls": 0,
// "unlockedSkinIds": []
// },
// "gameId": 0,
// "hasSimultaneousBans": false,
// "hasSimultaneousPicks": true,
// "isCustomGame": true,
// "isSpectating": false,
// "localPlayerCellId": 0,
// "lockedEventIndex": -1,
// "myTeam": [
// {
// "assignedPosition": "",
// "cellId": 0,
// "championId": 103,
// "championPickIntent": 0,
// "entitledFeatureType": "",
// "selectedSkinId": 103000,
// "spell1Id": 4,
// "spell2Id": 14,
// "summonerId": 68802321,
// "team": 1,
// "wardSkinId": -1
// }
// ],
// "rerollsRemaining": 0,
// "skipChampionSelect": false,
// "theirTeam": [],
// "timer": {
// "adjustedTimeLeftInPhase": 30589,
// "internalNowInEpochMs": 1619541990069,
// "isInfinite": false,
// "phase": "BAN_PICK",
// "totalTimeInPhase": 92848
// },
// "trades": []
// }

// POST /lol-perks/v1/pages
// {
// "autoModifiedSelections": [
// 0
// ],
// "current": true,
// "id": 0,
// "isActive": true,
// "isDeletable": true,
// "isEditable": true,
// "isValid": true,
// "lastModified": 0,
// "name": "Lee Sin",
// "order": 0,
// "primaryStyleId": 8000,
// "selectedPerkIds": [
// 8021,
// 8009,
// 8014,
// 8135,
// 8143,
// 9105,
// 5008,
// 5008,
// 5002
// ],
// "subStyleId": 8100
// }
