use crate::{
    error::Error, filter::internal_filter_engine_configuration, FilteredSearchEngines,
    SearchApiResult, SearchConfiguration, SearchUserEnvironment,
};
use error_support::handle_error;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default)]
struct SearchSelectorInner {
    configuration: SearchConfiguration,
}

pub struct SearchSelector(Mutex<SearchSelectorInner>);

impl SearchSelector {
    pub fn new() -> SearchSelector {
        Self(Mutex::new(SearchSelectorInner::default()))
    }

    #[handle_error(Error)]
    pub fn set_search_config(self: Arc<Self>, configuration: String) -> SearchApiResult<()> {
        self.0.lock().configuration = serde_json::from_str(&configuration)?;
        Ok(())
    }

    #[handle_error(Error)]
    pub fn filter_engine_configuration(
        &self,
        user_environment: SearchUserEnvironment,
    ) -> SearchApiResult<FilteredSearchEngines> {
        internal_filter_engine_configuration(user_environment, &self.0.lock().configuration.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SearchEngineDefinition, SearchEngineUrl, SearchEngineUrls, SearchUrlParam};

    const BASIC_CONFIG: &str = include_str!("./basic-config.json");

    #[test]
    fn it_returns_empty_for_no_matching_variants() {
        let selector = Arc::new(SearchSelector::new());
        Arc::clone(&selector).set_search_config(BASIC_CONFIG.to_string());

        let resp = selector
            .filter_engine_configuration(SearchUserEnvironment {
                locale: "fi".into(),
                region: "FR".into(),
                channel: String::new(),
                distribution_id: String::new(),
                experiment: String::new(),
                app_name: String::new(),
                version: String::new(),
            })
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
        let selector = Arc::new(SearchSelector::new());
        Arc::clone(&selector).set_search_config(BASIC_CONFIG.to_string());

        let resp = selector
            .filter_engine_configuration(SearchUserEnvironment {
                locale: String::from("en-US"),
                region: String::from("CA"),
                channel: String::new(),
                distribution_id: String::new(),
                experiment: String::new(),
                app_name: String::new(),
                version: String::new(),
            })
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
                                search_access_point: None
                            }]),
                            search_term_param_name: Some(String::from("q")),
                        },
                        suggestions: None,
                        trending: None,
                    },
                }],
                private_default_engine_id: None
            }
        );
    }
}
