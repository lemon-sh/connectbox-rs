//! API client library for the Compal CH7465CE, which is a cable modem provided by various European ISPs under the name Connect Box.

#![allow(clippy::missing_errors_doc)]
use std::{borrow::Cow, fmt::Display, sync::Arc};

pub use error::Error;
use models::PortForwardEntry;
use reqwest::{
    cookie::{CookieStore, Jar},
    header::HeaderValue,
    redirect::Policy,
    Client, Url,
};
use serde::de::DeserializeOwned;

mod error;
mod functions;
/// Data structures used by the library
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
            code,
            cookie_store,
            base_url,
            getter_url,
            setter_url,
            auto_reauth,
        })
    }

    fn cookie(&self, name: &str) -> Result<Option<String>> {
        let Some(cookies) = self.cookie_store.cookies(&self.base_url) else {
            return Ok(None)
        };
        let cookies = cookies.to_str()?;
        let Some(mut cookie_start) = cookies.find(&format!("{name}=")) else {
            return Ok(None)
        };
        cookie_start += name.len() + 1;
        let cookie_end = cookies[cookie_start..]
            .find(';')
            .map_or(cookies.len(), |p| p + cookie_start);
        Ok(Some(cookies[cookie_start..cookie_end].to_string()))
    }

    async fn xml_getter<T: DeserializeOwned>(&self, function: u32) -> Result<T> {
        let mut reauthed = false;
        loop {
            let session_token = self.cookie("sessionToken")?.ok_or(Error::NoSessionToken)?;
            let form: &[Field] = &[
                ("token".into(), session_token.into()),
                ("fun".into(), function.to_string().into()),
            ];
            tracing::debug!("Executing getter {function}");
            let req = self.http.post(self.getter_url.clone()).form(form);
            let resp = req.send().await?;
            if resp.status().is_redirection() {
                if self.auto_reauth && !reauthed {
                    reauthed = true;
                    tracing::info!(
                        "session <{}> has expired, attempting reauth",
                        self.cookie("SID")?.as_deref().unwrap_or("unknown")
                    );
                    self._login().await?;
                    continue;
                }
                return Err(Error::NotAuthorized);
            }
            return Ok(quick_xml::de::from_str(&resp.text().await?)?);
        }
    }

    async fn xml_setter(&self, function: u32, fields: Option<&[Field<'_, '_>]>) -> Result<String> {
        let mut reauthed = false;
        loop {
            let session_token = self.cookie("sessionToken")?.ok_or(Error::NoSessionToken)?;
            let mut form = vec![
                ("token".into(), session_token.into()),
                ("fun".into(), function.to_string().into()),
            ];
            if let Some(fields) = fields {
                for (key, value) in fields {
                    form.push((key.clone(), value.clone()));
                }
            }
            tracing::debug!("Executing setter {function} with body {form:?}");
            let req = self.http.post(self.setter_url.clone()).form(&form);
            let resp = req.send().await?;
            if resp.status().is_redirection() {
                if self.auto_reauth && !reauthed {
                    reauthed = true;
                    tracing::info!(
                        "session <{}> has expired, attempting reauth",
                        self.cookie("SID")?.as_deref().unwrap_or("unknown")
                    );
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
        let form: &[Field] = &[
            ("token".into(), session_token.into()),
            ("fun".into(), functions::LOGIN.to_string().into()),
            ("Username".into(), "NULL".into()),
            ("Password".into(), (&self.code).into()),
        ];
        let req = self.http.post(self.setter_url.clone()).form(form);
        let resp = req.send().await?;
        if resp.status().is_redirection() {
            if let Some(location) = resp.headers().get("Location").map(HeaderValue::to_str) {
                let location = location?;
                return if location == "../common_page/Access-denied.html" {
                    Err(Error::AccessDenied)
                } else {
                    Err(Error::UnexpectedRedirect(location.to_string()))
                };
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

    /// Log in to the router. This method must be called before using the client.
    pub async fn login(&self) -> Result<()> {
        // get the session cookie
        self.http
            .get(self.base_url.join("common_page/login.html")?)
            .send()
            .await?;

        self._login().await
    }

    /// Log out of the router.
    ///
    /// The Connect Box allows only one session at a time, thus you should call this method after you're done with using the client, so that other users can log in.
    pub async fn logout(&self) -> Result<()> {
        self.xml_setter(functions::LOGOUT, None).await?;
        tracing::info!(
            "session <{}>: logged out",
            self.cookie("SID")?.as_deref().unwrap_or("unknown")
        );
        Ok(())
    }

    /// Get all devices connected to the router.
    pub async fn devices(&self) -> Result<models::LanUserTable> {
        self.xml_getter(functions::LAN_TABLE).await
    }

    /// Get all port forwards.
    pub async fn port_forwards(&self) -> Result<models::PortForwards> {
        self.xml_getter(functions::FORWARDS).await
    }

    /// Toggle or remove port forwards.
    /// 
    /// This function accepts a predicate that will be called for every existing port forward. It should decide what to do with each port forward and return a [`PortForwardAction`].
    pub async fn edit_port_forwards<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(models::PortForwardEntry) -> PortForwardAction,
    {
        let mut instance = String::new();
        let mut enable = String::new();
        let mut delete = String::new();
        for entry in self.port_forwards().await?.entries {
            let id = entry.id;
            match f(entry) {
                PortForwardAction::Enable => {
                    enable.push_star("1");
                    delete.push_star("0");
                }
                PortForwardAction::Disable => {
                    enable.push_star("0");
                    delete.push_star("0");
                }
                PortForwardAction::Delete => {
                    enable.push_star("0");
                    delete.push_star("1");
                }
                PortForwardAction::Keep => continue,
            }
            instance.push_star(&id.to_string());
        }
        let fields = [
            ("action".into(), "apply".into()),
            ("instance".into(), instance.into()),
            ("local_IP".into(), "".into()),
            ("start_port".into(), "".into()),
            ("end_port".into(), "".into()),
            ("start_portIn".into(), "".into()),
            ("end_portIn".into(), "".into()),
            ("protocol".into(), "".into()),
            ("enable".into(), enable.into()),
            ("delete".into(), delete.into()),
            ("idd".into(), "".into()),
        ];
        let resp = self
            .xml_setter(functions::EDIT_FORWARDS, Some(&fields))
            .await?;
        if resp.is_empty() {
            Ok(())
        } else {
            Err(Error::Remote(resp))
        }
    }

    /// Add a port forward. The `id` field of the port is ignored.
    pub async fn add_port_forward(&self, port: &PortForwardEntry) -> Result<()> {
        let fields = [
            ("action".into(), "add".into()),
            ("instance".into(), "".into()),
            ("local_IP".into(), port.local_ip.to_string().into()),
            ("start_port".into(), port.start_port.to_string().into()),
            ("end_port".into(), port.end_port.to_string().into()),
            ("start_portIn".into(), port.start_port_in.to_string().into()),
            ("end_portIn".into(), port.end_port_in.to_string().into()),
            ("protocol".into(), port.protocol.id().to_string().into()),
            ("enable".into(), u8::from(port.enable).to_string().into()),
            ("delete".into(), "0".into()),
            ("idd".into(), "".into()),
        ];
        let resp = self
            .xml_setter(functions::EDIT_FORWARDS, Some(&fields))
            .await?;
        if resp.is_empty() {
            Ok(())
        } else {
            Err(Error::Remote(resp))
        }
    }
}

/// Specifies the action to perform with a given port forward. Used in conjunction with [`ConnectBox::edit_port_forwards`]
pub enum PortForwardAction {
    /// Don't do anything with the port forward
    Keep,
    /// Enable the port forward
    Enable,
    /// Disable the port forward
    Disable,
    /// Delete the port forward
    Delete,
}

trait StringExt {
    fn push_star(&mut self, string: &str);
}

impl StringExt for String {
    fn push_star(&mut self, string: &str) {
        if !self.is_empty() {
            self.push('*');
        }
        self.push_str(string);
    }
}
