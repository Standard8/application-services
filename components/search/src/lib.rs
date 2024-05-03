/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

mod error;
mod schema_types;

pub use error::SearchApiError;

pub(crate) use schema_types::{
    SearchAccessPointValues, SearchConfiguration, SearchEngineEnvironment, SearchEngineRecord,
    SearchEngineVariant, SearchRecords,
};
pub use schema_types::{SearchEngineUrl, SearchEngineUrls, SearchUrlParam};

pub mod selector;
pub use selector::{
    filter_engine_configuration, FilteredSearchEngines, SearchEngineDefinition,
    SearchUserEnvironment,
};

// pub(crate) type Result<T> = std::result::Result<T, error::Error>;
pub type SearchApiResult<T> = std::result::Result<T, error::SearchApiError>;

uniffi::include_scaffolding!("search");
