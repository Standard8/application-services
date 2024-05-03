/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchEngineEnvironment {
    #[serde(rename = "allRegionsAndLocales")]
    pub all_regions_and_locales: Option<bool>,
    pub distributions: Option<Vec<String>>,
    #[serde(rename = "excludedLocales")]
    pub excluded_locales: Option<Vec<String>>,
    #[serde(rename = "excludedRegions")]
    pub excluded_regions: Option<Vec<String>>,
    pub locales: Option<Vec<String>>,
    pub regions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SearchAccessPointValues {
    pub addressbar: Option<String>,
    pub contextmenu: Option<String>,
    pub homepage: Option<String>,
    pub newtab: Option<String>,
    pub searchbar: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SearchUrlParam {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "experimentConfig")]
    pub experiment_config: Option<String>,
    #[serde(rename = "searchAccessPoint")]
    pub search_access_point: Option<SearchAccessPointValues>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SearchEngineUrl {
    pub base: Option<String>,
    pub method: Option<String>,
    pub params: Option<Vec<SearchUrlParam>>,
    #[serde(rename = "searchTermParamName")]
    pub search_term_param_name: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SearchEngineUrls {
    pub search: SearchEngineUrl,
    pub suggestions: Option<SearchEngineUrl>,
    pub trending: Option<SearchEngineUrl>,
}

#[derive(Debug, Deserialize)]
pub struct SearchEngineBase {
    pub aliases: Option<Vec<String>>,
    pub charset: Option<String>,
    pub classification: String,
    pub name: String,
    #[serde(rename = "partnerCode")]
    pub partner_code: Option<String>,
    pub urls: SearchEngineUrls,
}

#[derive(Debug, Deserialize)]
pub struct SearchEngineVariant {
    pub environment: SearchEngineEnvironment,
    pub urls: Option<SearchEngineUrls>,
}

#[derive(Debug, Deserialize)]
pub struct SearchEngineRecord {
    pub base: SearchEngineBase,
    pub identifier: String,
    pub variants: Vec<SearchEngineVariant>,
}

#[derive(Debug, Deserialize)]
pub struct SearchDefaultEngines {
    #[serde(rename = "globalDefault")]
    pub global_default: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchEngineOrder {
    pub order: Vec<String>,
    // TODO
    // pub environment
}

#[derive(Debug, Deserialize)]
pub struct SearchEngineOrders {
    // Todo: fixme.
    pub orders: Vec<SearchEngineOrder>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "recordType")]
pub(crate) enum SearchRecords {
    #[serde(rename = "engine")]
    Engine(SearchEngineRecord),
    #[serde(rename = "defaultEngines")]
    DefaultEngines(SearchDefaultEngines),
    #[serde(rename = "engineOrders")]
    EngineOrders(SearchEngineOrders),
}

#[derive(Debug, Deserialize)]
pub(crate) struct SearchConfiguration {
    pub data: Vec<SearchRecords>,
}
