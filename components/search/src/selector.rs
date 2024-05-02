/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::{
    error::Error, SearchApiResult, SearchConfiguration, SearchEngineEnvironment,
    SearchEngineRecord, SearchEngineUrl, SearchEngineUrls, SearchEngineVariant, SearchRecords,
    SearchUrlParam,
};
use error_support::handle_error;

#[derive(Debug, PartialEq)]
pub struct SearchEngineDefinition {
    pub aliases: Option<Vec<String>>,
    pub classification: String,
    pub identifier: String,
    pub name: String,
    pub partner_code: Option<String>,
    pub telemetry_suffix: Option<String>,
    pub urls: SearchEngineUrls,
}

#[derive(Debug, PartialEq)]
pub struct FilteredSearchEngines {
    pub engines: Vec<SearchEngineDefinition>,
    pub private_default_engine_id: Option<String>,
}

/// Details of the user's environment.
/// Currently includes the following:
/// - `locale`: Users locale.
/// - `region`: Users region.
/// - `channel`: The update channel the application is running on.
/// - `distribution_id`: The distribution ID of the application.
/// - `experiment`: Any associated experiment id.
/// - `app_name`: The name of the application.
/// - `version`: The version of the application.#[derive(Debug)]
pub struct SearchUserEnvironment {
    pub locale: String,
    pub region: String,
    pub channel: String,
    pub distribution_id: String,
    pub experiment: String,
    pub app_name: String,
    pub version: String,
}

#[handle_error(Error)]
pub fn filter_engine_configuration(
    user_environment: SearchUserEnvironment,
    configuration: String,
) -> SearchApiResult<FilteredSearchEngines> {
    println!("{configuration}");
    let configuration: SearchConfiguration = serde_json::from_str(&configuration)?;
    let configuration = &configuration.data;

    let mut engines = Vec::new();

    for record in configuration {
        match record {
            SearchRecords::Engine(engine) => {
                let result = extract_engine_config(&user_environment, &engine);
                match result {
                    Some(result) => engines.push(result),
                    None => (),
                }
            }
            _ => (),
        }
    }

    Ok(FilteredSearchEngines {
        engines: engines,
        private_default_engine_id: None,
    })
}

fn extract_engine_config(
    user_environment: &SearchUserEnvironment,
    record: &SearchEngineRecord,
) -> Option<SearchEngineDefinition> {
    let variant = record
        .variants
        .iter()
        .rfind(|&v| matches_user_environment(v, user_environment));
    match variant {
        Some(variant) => {
            let base = &record.base;

            let mut engine_definition = SearchEngineDefinition {
                aliases: None,
                classification: base.classification.clone(),
                identifier: record.identifier.clone(),
                name: base.name.clone(),
                partner_code: base.partner_code.clone(),
                telemetry_suffix: None,
                urls: base.urls.clone(),
            };

            copy_variant_into(variant, &mut engine_definition);

            return Some(engine_definition);
        }
        None => None,
    }
}

fn copy_variant_into(
    variant: &SearchEngineVariant,
    engine_definition: &mut SearchEngineDefinition,
) {
    // TODO: Add more fields.
    match &variant.urls {
        Some(urls) => {
            copy_url_into(&urls.search, &mut engine_definition.urls.search);
        }
        None => (),
    }
}

fn copy_url_into(url: &SearchEngineUrl, engine_url: &mut SearchEngineUrl) {
    match &url.base {
        Some(base) => engine_url.base = Some(base.clone()),
        None => (),
    }
    match &url.method {
        Some(method) => engine_url.method = Some(method.clone()),
        None => (),
    }
    match &url.params {
        Some(params) => engine_url.params = Some(params.clone()),
        None => (),
    }
    match &url.search_term_param_name {
        Some(param_name) => engine_url.search_term_param_name = Some(param_name.clone()),
        None => (),
    }
}

fn matches_user_environment(
    variant: &SearchEngineVariant,
    user_environment: &SearchUserEnvironment,
) -> bool {
    // TODO: fill out.
    matches_region_and_locale(
        &user_environment.region,
        &user_environment.locale,
        &variant.environment,
    )
}

fn matches_region_and_locale(
    user_region: &String,
    user_locale: &String,
    config_env: &SearchEngineEnvironment,
) -> bool {
    // TODO: Excluded regions.

    if config_env
        .all_regions_and_locales
        .is_some_and(|v| v == true)
    {
        return true;
    }

    // When none of the regions and locales are set. This implies its available
    // everywhere.
    if config_env.all_regions_and_locales.is_none()
        && config_env.regions.is_none()
        && config_env.locales.is_none()
    {
        return true;
    }

    if does_config_include(&config_env.locales, user_locale)
        && does_config_include(&config_env.regions, user_region)
    {
        return true;
    }

    if does_config_include(&config_env.locales, user_locale) && config_env.regions.is_none() {
        return true;
    }

    if does_config_include(&config_env.regions, user_region) && config_env.locales.is_none() {
        return true;
    }

    false
}

fn does_config_include(config_array: &Option<Vec<String>>, compare_item: &String) -> bool {
    match config_array {
        Some(array) => array.iter().any(|item| item == compare_item),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_CONFIG: &str = include_str!("./basic-config.json");

    #[test]
    fn it_returns_empty_for_no_matching_variants() {
        let resp = filter_engine_configuration(
            SearchUserEnvironment {
                locale: "fi".into(),
                region: "FR".into(),
                channel: String::new(),
                distribution_id: String::new(),
                experiment: String::new(),
                app_name: String::new(),
                version: String::new(),
            },
            BASIC_CONFIG.to_string(),
        )
        .unwrap();
        assert_eq!(
            resp,
            FilteredSearchEngines {
                engines: vec![],
                private_default_engine_id: None
            }
        );
    }

    #[test]
    fn it_matches_and_applies_the_last_variant() {
        let resp = filter_engine_configuration(
            SearchUserEnvironment {
                locale: String::from("en-US"),
                region: String::from("CA"),
                channel: String::new(),
                distribution_id: String::new(),
                experiment: String::new(),
                app_name: String::new(),
                version: String::new(),
            },
            BASIC_CONFIG.to_string(),
        )
        .unwrap();
        assert_eq!(
            resp,
            FilteredSearchEngines {
                engines: vec![SearchEngineDefinition {
                    aliases: None,
                    classification: String::from("unknown"),
                    identifier: String::from("engine-1"),
                    name: String::from("engine"),
                    partner_code: None,
                    telemetry_suffix: None,
                    urls: SearchEngineUrls {
                        search: SearchEngineUrl {
                            base: Some(String::from("https://example.com")),
                            method: None,
                            params: Some(vec![SearchUrlParam {
                                name: String::from("partner-code"),
                                value: Some(String::from("foo")),
                                experiment_config: None,
                            }]),
                            search_term_param_name: Some(String::from("q")),
                        },
                        suggestion: None,
                        trending: None,
                    },
                }],
                private_default_engine_id: None
            }
        );
    }
}
