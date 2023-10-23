use std::error::Error;
use curl::easy::{Easy2, Handler, List, WriteError};
use log::info;
use serde::de::DeserializeOwned;

pub struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

impl Collector {
    pub fn parse_data<T>(&self) -> Result<T, Box<dyn Error>>
        where
            T: DeserializeOwned,
    {
        Ok(serde_json::from_slice(&self.0)?)
    }

    pub fn get_text(&self) -> Result<String, Box<dyn Error>> {
        Ok(String::from_utf8(self.0.clone())?)
    }
}

pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

pub struct Request {
    pub url: String,
    pub headers: Option<Vec<String>>,
    pub method: HttpMethod,
    pub body: Option<Vec<u8>>,
    pub easy: Option<Easy2<Collector>>
}

impl Request {
    pub fn new(url: String, headers: Option<Vec<String>>, method: HttpMethod, body: Option<Vec<u8>>) -> Self {
        Request {
            url,
            headers,
            method,
            body,
            easy: None,
        }
    }

    pub fn redirect_url(&mut self) -> Result<String, Box<dyn Error>>{
        return if let Some(easy) = &mut self.easy {
            match easy.effective_url_bytes() {
                Ok(r) => {
                    if let Some(url) = r{
                        info!("follow from: {:?}", String::from_utf8_lossy(url));
                    }
                }
                Err(_) => {}
            }
            if let Some(url) = easy.redirect_url()? {
                Ok(url.to_string())
            }else {
                Err("Unable to get redirect url".into())
            }
        } else {
            Err("Request hasn't been executed".into())
        }

    }
    pub fn get_response(&mut self) -> Result<&Collector, Box<dyn Error>>{
        if let Some(easy) = &self.easy {
            let data = easy.get_ref();
            Ok(data)
        }else{
            return Err("Request hasn't been executed".into())
        }
    }
    pub fn get_cookie(&mut self) -> Result<List, Box<dyn Error>> {
        return if let Some(easy) = &mut self.easy {
            let cookie = easy.cookies()?;
            Ok(cookie)
        } else {
            Err("Request hasn't been executed".into())
        }
    }


    pub fn execute(&mut self, cookie: &str) -> Result<(), Box<dyn Error>> {
        info!("request: {}", self.url);
        let mut easy = Easy2::new(Collector(vec![]));
        easy.cookie_session(true)?;
        easy.cookie_list("")?;
        easy.cookie(cookie)?;
        easy.useragent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15")?;
        if let Some(_headers) = &self.headers {
            let mut list = List::new();
            for header in _headers.iter() {
                list.append(header.as_str())?;
            }
            easy.http_headers(list)?;
        }
        easy.url(self.url.as_str())?;
        match self.method {
            HttpMethod::GET => {
                easy.get(true)?;
            }
            HttpMethod::POST => {
                easy.post(true)?;
                if let Some(body_data) = &self.body {
                    easy.post_fields_copy(body_data)?;

                }
            }
            HttpMethod::PUT => {
                easy.put(true)?;
                if let Some(body_data) = &self.body {
                    easy.post_fields_copy(body_data)?;
                }
            }
            HttpMethod::DELETE => {
                easy.custom_request("DELETE")?;
            }
        }
        easy.perform()?;
        self.easy = Some(easy);
        Ok(())
    }
}
