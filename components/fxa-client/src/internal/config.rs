/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::http_client;
use crate::{FxaConfig, Result};
use serde_derive::{Deserialize, Serialize};
use std::{cell::RefCell, sync::Arc};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    content_url: String,
    token_server_url_override: Option<String>,
    pub client_id: String,
    pub redirect_uri: String,
    // RemoteConfig is lazily fetched from the server.
    #[serde(skip)]
    remote_config: RefCell<Option<Arc<RemoteConfig>>>,
}

/// `RemoteConfig` struct stores configuration values from the FxA
/// `/.well-known/fxa-client-configuration` and the
/// `/.well-known/openid-configuration` endpoints.
#[derive(Debug)]
// allow(dead_code) since we want the struct to match the API data, even if some fields aren't
// currently used.
#[allow(dead_code)]
pub struct RemoteConfig {
    auth_url: String,
    oauth_url: String,
    profile_url: String,
    token_server_endpoint_url: String,
    authorization_endpoint: String,
    issuer: String,
    jwks_uri: String,
    token_endpoint: String,
    userinfo_endpoint: String,
    introspection_endpoint: String,
}

pub(crate) const CONTENT_URL_RELEASE: &str = "https://accounts.firefox.com";
pub(crate) const CONTENT_URL_CHINA: &str = "https://accounts.firefox.com.cn";

impl Config {
    fn remote_config(&self) -> Result<Arc<RemoteConfig>> {
        if let Some(remote_config) = self.remote_config.borrow().clone() {
            return Ok(remote_config);
        }

        let client_config = http_client::fxa_client_configuration(self.client_config_url()?)?;
        let openid_config = http_client::openid_configuration(self.openid_config_url()?)?;

        let remote_config = self.set_remote_config(RemoteConfig {
            auth_url: format!("{}/", client_config.auth_server_base_url),
            oauth_url: format!("{}/", client_config.oauth_server_base_url),
            profile_url: format!("{}/", client_config.profile_server_base_url),
            token_server_endpoint_url: format!("{}/", client_config.sync_tokenserver_base_url),
            authorization_endpoint: openid_config.authorization_endpoint,
            issuer: openid_config.issuer,
            jwks_uri: openid_config.jwks_uri,
            // TODO: bring back openid token endpoint once https://github.com/mozilla/fxa/issues/453 has been resolved
            // and the openid response has been switched to the new endpoint.
            // token_endpoint: openid_config.token_endpoint,
            token_endpoint: format!("{}/v1/oauth/token", client_config.auth_server_base_url),
            userinfo_endpoint: openid_config.userinfo_endpoint,
            introspection_endpoint: openid_config.introspection_endpoint,
        });
        Ok(remote_config)
    }

    fn set_remote_config(&self, remote_config: RemoteConfig) -> Arc<RemoteConfig> {
        let rc = Arc::new(remote_config);
        let result = rc.clone();
        self.remote_config.replace(Some(rc));
        result
    }

    pub fn content_url(&self) -> Result<Url> {
        Url::parse(&self.content_url).map_err(Into::into)
    }

    pub fn content_url_path(&self, path: &str) -> Result<Url> {
        self.content_url()?.join(path).map_err(Into::into)
    }

    pub fn client_config_url(&self) -> Result<Url> {
        self.content_url_path(".well-known/fxa-client-configuration")
    }

    pub fn openid_config_url(&self) -> Result<Url> {
        self.content_url_path(".well-known/openid-configuration")
    }

    pub fn connect_another_device_url(&self) -> Result<Url> {
        self.content_url_path("connect_another_device")
    }

    pub fn pair_url(&self) -> Result<Url> {
        self.content_url_path("pair")
    }

    pub fn pair_supp_url(&self) -> Result<Url> {
        self.content_url_path("pair/supp")
    }

    pub fn oauth_force_auth_url(&self) -> Result<Url> {
        self.content_url_path("oauth/force_auth")
    }

    pub fn settings_url(&self) -> Result<Url> {
        self.content_url_path("settings")
    }

    pub fn settings_clients_url(&self) -> Result<Url> {
        self.content_url_path("settings/clients")
    }

    pub fn auth_url(&self) -> Result<Url> {
        Url::parse(&self.remote_config()?.auth_url).map_err(Into::into)
    }

    pub fn auth_url_path(&self, path: &str) -> Result<Url> {
        self.auth_url()?.join(path).map_err(Into::into)
    }

    pub fn oauth_url(&self) -> Result<Url> {
        Url::parse(&self.remote_config()?.oauth_url).map_err(Into::into)
    }

    pub fn oauth_url_path(&self, path: &str) -> Result<Url> {
        self.oauth_url()?.join(path).map_err(Into::into)
    }

    pub fn token_server_endpoint_url(&self) -> Result<Url> {
        if let Some(token_server_url_override) = &self.token_server_url_override {
            return Ok(Url::parse(token_server_url_override)?);
        }
        Ok(Url::parse(
            &self.remote_config()?.token_server_endpoint_url,
        )?)
    }

    pub fn authorization_endpoint(&self) -> Result<Url> {
        Url::parse(&self.remote_config()?.authorization_endpoint).map_err(Into::into)
    }

    pub fn token_endpoint(&self) -> Result<Url> {
        Url::parse(&self.remote_config()?.token_endpoint).map_err(Into::into)
    }

    pub fn introspection_endpoint(&self) -> Result<Url> {
        Url::parse(&self.remote_config()?.introspection_endpoint).map_err(Into::into)
    }

    pub fn userinfo_endpoint(&self) -> Result<Url> {
        Url::parse(&self.remote_config()?.userinfo_endpoint).map_err(Into::into)
    }

    fn normalize_token_server_url(token_server_url_override: &str) -> String {
        // In self-hosting setups it is common to specify the `/1.0/sync/1.5` suffix on the
        // tokenserver URL. Accept and strip this form as a convenience for users.
        // (ideally we'd use `strip_suffix`, but we currently target a rust version
        // where this doesn't exist - `trim_end_matches` will repeatedly remove
        // the suffix, but that seems fine for this use-case)
        token_server_url_override
            .trim_end_matches("/1.0/sync/1.5")
            .to_owned()
    }
}

impl From<FxaConfig> for Config {
    fn from(fxa_config: FxaConfig) -> Self {
        let content_url = fxa_config.server.content_url().to_string();
        let token_server_url_override = fxa_config
            .token_server_url_override
            .as_deref()
            .map(Self::normalize_token_server_url);

        Self {
            content_url,
            client_id: fxa_config.client_id,
            redirect_uri: fxa_config.redirect_uri,
            token_server_url_override,
            remote_config: RefCell::new(None),
        }
    }
}

#[cfg(test)]
/// Testing functionality
impl Config {
    pub fn release(client_id: &str, redirect_uri: &str) -> Self {
        Self::new(CONTENT_URL_RELEASE, client_id, redirect_uri)
    }

    pub fn stable_dev(client_id: &str, redirect_uri: &str) -> Self {
        Self::new("https://stable.dev.lcip.org", client_id, redirect_uri)
    }

    pub fn china(client_id: &str, redirect_uri: &str) -> Self {
        Self::new(CONTENT_URL_CHINA, client_id, redirect_uri)
    }

    pub fn new(content_url: &str, client_id: &str, redirect_uri: &str) -> Self {
        Self {
            content_url: content_url.to_string(),
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            remote_config: RefCell::new(None),
            token_server_url_override: None,
        }
    }

    /// Override the token server URL that would otherwise be provided by the
    /// FxA .well-known/fxa-client-configuration endpoint.
    /// This is used by self-hosters that still use the product FxA servers
    /// for authentication purposes but use their own Sync storage backend.
    pub fn override_token_server_url<'a>(
        &'a mut self,
        token_server_url_override: &str,
    ) -> &'a mut Self {
        self.token_server_url_override =
            Some(Self::normalize_token_server_url(token_server_url_override));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        let remote_config = RemoteConfig {
            auth_url: "https://stable.dev.lcip.org/auth/".to_string(),
            oauth_url: "https://oauth-stable.dev.lcip.org/".to_string(),
            profile_url: "https://stable.dev.lcip.org/profile/".to_string(),
            token_server_endpoint_url: "https://stable.dev.lcip.org/syncserver/token/1.0/sync/1.5"
                .to_string(),
            authorization_endpoint: "https://oauth-stable.dev.lcip.org/v1/authorization"
                .to_string(),
            issuer: "https://dev.lcip.org/".to_string(),
            jwks_uri: "https://oauth-stable.dev.lcip.org/v1/jwks".to_string(),
            token_endpoint: "https://stable.dev.lcip.org/auth/v1/oauth/token".to_string(),
            introspection_endpoint: "https://oauth-stable.dev.lcip.org/v1/introspect".to_string(),
            userinfo_endpoint: "https://stable.dev.lcip.org/profile/v1/profile".to_string(),
        };

        let config = Config {
            content_url: "https://stable.dev.lcip.org/".to_string(),
            remote_config: RefCell::new(Some(Arc::new(remote_config))),
            client_id: "263ceaa5546dce83".to_string(),
            redirect_uri: "https://127.0.0.1:8080".to_string(),
            token_server_url_override: None,
        };
        assert_eq!(
            config.auth_url_path("v1/account/keys").unwrap().to_string(),
            "https://stable.dev.lcip.org/auth/v1/account/keys"
        );
        assert_eq!(
            config.oauth_url_path("v1/token").unwrap().to_string(),
            "https://oauth-stable.dev.lcip.org/v1/token"
        );
        assert_eq!(
            config.content_url_path("oauth/signin").unwrap().to_string(),
            "https://stable.dev.lcip.org/oauth/signin"
        );
        assert_eq!(
            config.token_server_endpoint_url().unwrap().to_string(),
            "https://stable.dev.lcip.org/syncserver/token/1.0/sync/1.5"
        );

        assert_eq!(
            config.token_endpoint().unwrap().to_string(),
            "https://stable.dev.lcip.org/auth/v1/oauth/token"
        );

        assert_eq!(
            config.introspection_endpoint().unwrap().to_string(),
            "https://oauth-stable.dev.lcip.org/v1/introspect"
        );
    }

    #[test]
    fn test_tokenserver_url_override() {
        let remote_config = RemoteConfig {
            auth_url: "https://stable.dev.lcip.org/auth/".to_string(),
            oauth_url: "https://oauth-stable.dev.lcip.org/".to_string(),
            profile_url: "https://stable.dev.lcip.org/profile/".to_string(),
            token_server_endpoint_url: "https://stable.dev.lcip.org/syncserver/token/1.0/sync/1.5"
                .to_string(),
            authorization_endpoint: "https://oauth-stable.dev.lcip.org/v1/authorization"
                .to_string(),
            issuer: "https://dev.lcip.org/".to_string(),
            jwks_uri: "https://oauth-stable.dev.lcip.org/v1/jwks".to_string(),
            token_endpoint: "https://stable.dev.lcip.org/auth/v1/oauth/token".to_string(),
            introspection_endpoint: "https://oauth-stable.dev.lcip.org/v1/introspect".to_string(),
            userinfo_endpoint: "https://stable.dev.lcip.org/profile/v1/profile".to_string(),
        };

        let mut config = Config {
            content_url: "https://stable.dev.lcip.org/".to_string(),
            remote_config: RefCell::new(Some(Arc::new(remote_config))),
            client_id: "263ceaa5546dce83".to_string(),
            redirect_uri: "https://127.0.0.1:8080".to_string(),
            token_server_url_override: None,
        };

        config.override_token_server_url("https://foo.bar");

        assert_eq!(
            config.token_server_endpoint_url().unwrap().to_string(),
            "https://foo.bar/"
        );
    }

    #[test]
    fn test_tokenserver_url_override_strips_sync_service_prefix() {
        let remote_config = RemoteConfig {
            auth_url: "https://stable.dev.lcip.org/auth/".to_string(),
            oauth_url: "https://oauth-stable.dev.lcip.org/".to_string(),
            profile_url: "https://stable.dev.lcip.org/profile/".to_string(),
            token_server_endpoint_url: "https://stable.dev.lcip.org/syncserver/token/".to_string(),
            authorization_endpoint: "https://oauth-stable.dev.lcip.org/v1/authorization"
                .to_string(),
            issuer: "https://dev.lcip.org/".to_string(),
            jwks_uri: "https://oauth-stable.dev.lcip.org/v1/jwks".to_string(),
            token_endpoint: "https://stable.dev.lcip.org/auth/v1/oauth/token".to_string(),
            introspection_endpoint: "https://oauth-stable.dev.lcip.org/v1/introspect".to_string(),
            userinfo_endpoint: "https://stable.dev.lcip.org/profile/v1/profile".to_string(),
        };

        let mut config = Config {
            content_url: "https://stable.dev.lcip.org/".to_string(),
            remote_config: RefCell::new(Some(Arc::new(remote_config))),
            client_id: "263ceaa5546dce83".to_string(),
            redirect_uri: "https://127.0.0.1:8080".to_string(),
            token_server_url_override: None,
        };

        config.override_token_server_url("https://foo.bar/prefix/1.0/sync/1.5");
        assert_eq!(
            config.token_server_endpoint_url().unwrap().to_string(),
            "https://foo.bar/prefix"
        );

        config.override_token_server_url("https://foo.bar/prefix-1.0/sync/1.5");
        assert_eq!(
            config.token_server_endpoint_url().unwrap().to_string(),
            "https://foo.bar/prefix-1.0/sync/1.5"
        );

        config.override_token_server_url("https://foo.bar/1.0/sync/1.5/foobar");
        assert_eq!(
            config.token_server_endpoint_url().unwrap().to_string(),
            "https://foo.bar/1.0/sync/1.5/foobar"
        );
    }
}
