//! API client library for the Compal CH7465CE, which is a cable modem provided by various European ISPs under the name Connect Box.

use std::{borrow::Cow, fmt::Display, sync::Arc};

pub use error::Error;
use reqwest::{
    cookie::{CookieStore, Jar},
    redirect::Policy,
    Client, Url, header::HeaderValue,
};
use serde::de::DeserializeOwned;

mod error;
mod functions;
/// Data structures used by the library.
pub mod models;

/// A Result type based on the library's Error
pub type Result<T> = std::result::Result<T, error::Error>;

type Field<'a, 'b> = (Cow<'a, str>, Cow<'b, str>);

/// The entry point of the library - the API client
pub struct ConnectBox {
    http: Client,
    code: String,
    cookie_store: Arc<Jar>,
    base_url: Url,
    getter_url: Url,
    setter_url: Url,
    auto_reauth: bool,
}

impl ConnectBox {
    /// Create a new client associated with the specified address. You must call [`login`](Self::login()) before use.
    /// * `code` - the router password
    /// * `auto_reauth` - whether to automatically re-authenticate when the session expires
    pub fn new(address: impl Display, code: String, auto_reauth: bool) -> Result<Self> {
        let cookie_store = Arc::new(Jar::default());
        let http = Client::builder()
            .user_agent("Mozilla/5.0")
            .redirect(Policy::none())
            .cookie_provider(cookie_store.clone())
            .build()?;
        let base_url: Url = format!("http://{address}/").parse()?;
        let getter_url = base_url.join("xml/getter.xml")?;
        let setter_url = base_url.join("xml/setter.xml")?;
        Ok(ConnectBox {
            http,
            cookie_store,
            base_url,
            getter_url,
            setter_url,
            code,
            auto_reauth,
        })
    }

    fn cookie<'a>(&self, name: &str) -> Result<Option<String>> {
        let Some(cookies) = self.cookie_store.cookies(&self.base_url) else {
            return Ok(None)
        };
        let cookies = cookies.to_str()?;
        let Some(mut cookie_start) = cookies.find(&format!("{name}=")) else {
            return Ok(None)
        };
        cookie_start += name.len() + 1;
        let cookie_end = cookies[cookie_start..]
            .find(";")
            .map(|p| p + cookie_start)
            .unwrap_or(cookies.len());
        Ok(Some(cookies[cookie_start..cookie_end].to_string()))
    }

    async fn xml_getter<T: DeserializeOwned>(&self, function: u32) -> Result<T> {
        let mut reauthed = false;
        loop {
            let session_token = self.cookie("sessionToken")?.ok_or(Error::NoSessionToken)?;
            let form: Vec<Field> = vec![
                ("token".into(), session_token.into()),
                ("fun".into(), function.to_string().into()),
            ];
            let req = self.http.post(self.getter_url.clone()).form(&form);
            let resp = req.send().await?;
            if resp.status().is_redirection() {
                if self.auto_reauth && !reauthed {
                    reauthed = true;
                    tracing::info!("session <{}> has expired, attempting reauth", self.cookie("SID")?.as_deref().unwrap_or("unknown"));
                    self._login().await?;
                    continue;
                }
                return Err(Error::NotAuthorized);
            }
            return Ok(quick_xml::de::from_str(&resp.text().await?)?);
        }
    }

    async fn xml_setter(
        &self,
        function: u32,
        fields: Option<impl AsRef<[Field<'_, '_>]>>,
    ) -> Result<String> {
        let mut reauthed = false;
        loop {
            let session_token = self.cookie("sessionToken")?.ok_or(Error::NoSessionToken)?;
            let mut form: Vec<(Cow<str>, Cow<str>)> = vec![
                ("token".into(), session_token.into()),
                ("fun".into(), function.to_string().into()),
            ];
            if let Some(fields) = &fields {
                for (key, value) in fields.as_ref() {
                    form.push((key.clone(), value.clone()));
                }
            }
            let req = self.http.post(self.setter_url.clone()).form(&form);
            let resp = req.send().await?;
            if resp.status().is_redirection() {
                if self.auto_reauth && !reauthed {
                    reauthed = true;
                    tracing::info!("session <{}> has expired, attempting reauth", self.cookie("SID")?.as_deref().unwrap_or("unknown"));
                    self._login().await?;
                    continue;
                }
                return Err(Error::NotAuthorized);
            }
            return Ok(resp.text().await?);
        }
    }

    async fn _login(&self) -> Result<()> {
        let session_token = self.cookie("sessionToken")?.ok_or(Error::NoSessionToken)?;
        let form: Vec<(Cow<str>, Cow<str>)> = vec![
            ("token".into(), session_token.into()),
            ("fun".into(), functions::LOGIN.to_string().into()),
            ("Username".into(), "NULL".into()),
            ("Password".into(), (&self.code).into()),
        ];
        let req = self.http.post(self.setter_url.clone()).form(&form);
        let resp = req.send().await?;
        if resp.status().is_redirection() {
            if let Some(location) = resp.headers().get("Location").map(HeaderValue::to_str) {
                let location = location?;
                return if location == "../common_page/Access-denied.html" {
                    Err(Error::AccessDenied)
                } else {
                    Err(Error::UnexpectedRedirect(location.to_string()))
                }
            }
        }
        let resp_text = resp.text().await?;
        if resp_text == "idloginincorrect" {
            return Err(Error::IncorrectCode);
        }
        let sid = resp_text
            .strip_prefix("successful;SID=")
            .ok_or_else(|| Error::UnexpectedResponse(resp_text.clone()))?;
        tracing::info!("session <{sid}>: logged in successfully");
        self.cookie_store
            .add_cookie_str(&format!("SID={sid}"), &self.base_url);

        Ok(())
    }

    /// Login to the router. This method must be called before using the client.
    pub async fn login(&self) -> Result<()> {
        // get the session cookie
        self.http
            .get(self.base_url.join("common_page/login.html")?)
            .send()
            .await?;

        self._login().await
    }

    /// Get all devices connected to the router.
    pub async fn devices(&self) -> Result<models::LanUserTable> {
        self.xml_getter(functions::LAN_TABLE).await
    }

    /// Get all port forwarding entries.
    pub async fn port_forwards(&self) -> Result<models::PortForwards> {
        self.xml_getter(functions::FORWARDS).await
    }
}
