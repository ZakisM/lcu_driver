use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::endpoints::EndpointInfo;

const PERKS_URL: &str = "/lol-perks/v1";

#[allow(unused)]
pub enum PerksEndpoint {
    Inventory,
    Pages(Method, Option<String>),
    PagesId(Method, isize),
}

impl PerksEndpoint {
    pub fn info(&self) -> EndpointInfo {
        match self {
            PerksEndpoint::Inventory => EndpointInfo {
                url: format!("{}/inventory", PERKS_URL),
                method: Method::GET,
                headers: None,
                body: None,
            },
            PerksEndpoint::Pages(method, body) => EndpointInfo {
                url: format!("{}/pages", PERKS_URL),
                method: method.to_owned(),
                headers: None,
                body: body.to_owned(),
            },
            PerksEndpoint::PagesId(method, id) => EndpointInfo {
                url: format!("{}/pages/{}", PERKS_URL, id),
                method: method.to_owned(),
                headers: None,
                body: None,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerksInventory {
    pub owned_page_count: isize,
}

#[derive(Debug)]
pub struct PerksPages {
    pub pages: Vec<PerksPage>,
}

#[derive(Debug, Serialize, Deserialize, Eq)]
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

impl std::cmp::PartialEq for PerksPage {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.primary_style_id == other.primary_style_id
            && self.selected_perk_ids == other.selected_perk_ids
            && self.sub_style_id == other.sub_style_id
    }
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
