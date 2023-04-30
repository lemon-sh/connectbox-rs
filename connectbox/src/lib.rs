//! API client library for the Compal CH7465CE, which is a cable modem provided by various European ISPs under the name Connect Box.

use std::{borrow::Cow, fmt::Display, sync::Arc};

use error::Error;
use reqwest::{
    cookie::{CookieStore, Jar},
    redirect::Policy,
    Client, Url,
};
use serde::de::DeserializeOwned;

pub mod error;
mod functions;
pub mod models;

pub(crate) type Result<T> = std::result::Result<T, error::Error>;

type Field<'a, 'b> = (Cow<'a, str>, Cow<'b, str>);

/// The entry point of the library - the API client
pub struct ConnectBox {
    http: Client,
    code: String,
    cookie_store: Arc<Jar>,
    base_url: Url,
    getter_url: Url,
    setter_url: Url,
    auto_reauth: bool
}

impl ConnectBox {
    pub fn new(address: impl Display, code: String) -> Result<Self> {
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
            auto_reauth: false
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
                    continue;
                }
                return Err(Error::NotAuthorized)
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
                    continue;
                }
                return Err(Error::NotAuthorized)
            }
            return Ok(resp.text().await?);
        }
    }

    async fn _login(&self) -> Result<()> {
        let fields = vec![
            ("Username".into(), "NULL".into()),
            ("Password".into(), (&self.code).into()),
        ];
        let response = self.xml_setter(functions::LOGIN, Some(fields)).await?;
        if response == "idloginincorrect" {
            return Err(Error::IncorrectCode);
        }
        let sid = response
            .strip_prefix("successful;SID=")
            .ok_or_else(|| Error::UnexpectedResponse(response.clone()))?;
        self.cookie_store
            .add_cookie_str(&format!("SID={sid}"), &self.base_url);

        Ok(())
    }

    pub async fn login(&self) -> Result<()> {
        // get the session cookie
        self.http
            .get(self.base_url.join("common_page/login.html")?)
            .send()
            .await?;

        self._login().await
    }

    pub async fn get_devices(&self) -> Result<models::LanUserTable> {
        self.xml_getter(functions::LAN_TABLE).await
    }
}
