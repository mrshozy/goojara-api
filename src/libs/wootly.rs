use std::error::Error;
use std::fmt::format;
use curl::easy::List;
use log::error;
use regex::Regex;
use crate::libs::utils::request_client::{Collector, HttpMethod, Request};

#[derive(Debug, Clone)]
struct Parser;

impl Parser{

    fn get_url(&self, html: &str) -> Result<(String, String), Box<dyn Error>>{
        let regex = Regex::new(r#"var\svd.+?,cn.+?,cv.+?,dd.+?,tk.+?";"#).unwrap();
        if let Some(capture) = regex.captures(html){
            let data = capture.get(0).unwrap().as_str();
            let mut vd = "";
            let mut cn = "";
            let mut cv = "";
            let mut dd = "";
            let mut tk = "";
            for part in data.split(',') {
                let parts: Vec<&str> = part.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "var vd" => vd = parts[1],
                        "cn" => cn = parts[1],
                        "cv" => cv = parts[1],
                        "dd" => dd = parts[1],
                        "tk" => tk = parts[1],
                        _ => {}
                    }
                }
            }
            Ok((format!("https://www.wootly.ch/grabd?t={}&id={}", tk.trim_matches(|p| p=='"' || p==';'), vd.trim_matches(|p| p=='"' || p==';')),
                format!("{}={};", cn.trim_matches(|p| p=='"' || p==';'), cv.trim_matches(|p| p=='"' || p==';'))
            ))
        }else{
            Err("Unable to get url".into())
        }
    }
}
#[derive(Debug, Clone)]
pub struct Wootly{
    url:String,
    cookie:String,
    parser:Parser
}

impl Wootly{
    pub fn new(url: &str) -> Self{
        Self{
            url: url.to_string(),
            cookie: String::new(),
            parser: Parser{}
        }
    }
    fn get_real_url(&self, url:&str) -> Result<String, Box<dyn Error>>{
        let headers = vec!["Referer: https://www.wootly.ch/".to_string()];
        let request = &mut Request::new(url.to_string(), Some(headers), HttpMethod::POST, Some("qdf=1".to_string().into_bytes()));
        match request.execute("") {
            Ok(_) => {
                match request.get_response() {
                    Ok(_) => {
                        return request.redirect_url();
                    },
                    Err(e) => {
                        error!("{}", e)
                    }
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        }
        Err("Failed to get the final link".into())
    }
    fn get_data_source(&self, url: &str, cookies: &str) -> Result<String, Box<dyn Error>>{
        let request = &mut Request::new(url.to_string(), None, HttpMethod::POST, Some("qdf=1".to_string().into_bytes()));
        match request.execute(cookies) {
            Ok(_) => {
                match request.get_response() {
                    Ok(_) => {
                       return self.get_real_url(request.redirect_url()?.as_str())
                    }
                    Err(e) => {
                        error!("{}", e)
                    }
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        }
        Err("Unknown error occurred".into())
    }
    pub fn get_url(&self) -> Result<String, Box<dyn Error>>{
        let request = &mut Request::new(self.url.clone(), None, HttpMethod::POST, Some("qdf=1".to_string().into_bytes()));
        match request.execute(self.cookie.as_str()) {
            Ok(()) => {
                match request.get_response() {
                    Ok(data) => {
                        let html = data.get_text()?;
                        match &mut self.parser.get_url(html.as_str()) {
                            Ok((source, cookie)) => {
                               let cookie = match request.get_cookie() {
                                    Ok(_cookie) => {
                                        match _cookie.iter().last() {
                                            None => cookie.as_str().to_string(),
                                            Some(bytes) => {
                                                let cookie_string = String::from_utf8_lossy(bytes).to_string();
                                                let parse_cookie = cookie_string.split("0\t").last().ok_or("unable to parse cookie")?.replace("\t", "=");
                                                format!("{} {}", cookie, parse_cookie)
                                            }
                                        }
                                    }
                                    Err(_) => cookie.as_str().to_string()
                                };
                                let request = &mut Request::new(source.clone(), None, HttpMethod::GET, None);
                                request.execute(cookie.as_str())?;
                                let url = request.get_response()?.get_text()?;
                                let redirected = self.get_data_source(url.as_str(), cookie.as_str())?;
                                return Ok(redirected)
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {
                    }
                }
            }
            Err(_) => {}
        }
        Err("Unknown error occurred".into())
    }
}