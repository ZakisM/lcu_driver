use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::endpoints::EndpointInfo;

const PERKS_URL: &str = "/lol-perks/v1";

#[allow(unused)]
pub enum PerksEndpoint {
    Pages(Method, Option<String>),
}

impl PerksEndpoint {
    pub fn info(&self) -> EndpointInfo {
        match self {
            PerksEndpoint::Pages(method, body) => EndpointInfo {
                url: format!("{}/pages", PERKS_URL),
                method: method.to_owned(),
                headers: None,
                body: body.to_owned(),
            },
        }
    }
}

#[derive(Debug)]
pub struct PerksPages {
    pages: Vec<PerksPage>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerksPage {
    pub auto_modified_selections: Vec<isize>,
    pub current: bool,
    pub id: isize,
    pub is_active: bool,
    pub is_deletable: bool,
    pub is_editable: bool,
    pub is_valid: bool,
    pub last_modified: isize,
    pub name: String,
    pub order: isize,
    pub primary_style_id: isize,
    pub selected_perk_ids: Vec<isize>,
    pub sub_style_id: isize,
}

impl Default for PerksPage {
    fn default() -> Self {
        Self {
            auto_modified_selections: Vec::new(),
            current: true,
            id: 0,
            is_active: true,
            is_deletable: true,
            is_editable: true,
            is_valid: true,
            last_modified: 0,
            name: "".to_owned(),
            order: 0,
            primary_style_id: 0,
            selected_perk_ids: Vec::new(),
            sub_style_id: 0,
        }
    }
}
