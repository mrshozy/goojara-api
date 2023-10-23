use std::error::Error;
use std::thread::scope;
use actix_web::cookie::Cookie;
use chrono::format::format;
use chrono::Local;
use curl::easy::List;
use log::{error, info};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::libs::utils::hashing::SimpleHash;
use crate::libs::utils::request_client::{Collector, HttpMethod, Request};
use crate::libs::wootly::Wootly;

#[derive(Clone, Debug)]
pub struct Goojara {
    parser: Parser,
    cookies: String
}

#[derive(Clone, Debug)]
pub struct Parser {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Movie {
    url: String,
    title: String,
    img: String,
    quality: String,
    year: String,
}

#[derive(Clone, Debug)]
pub enum Genre {
    ACTION,
    ADVENTURE,
    COMEDY,
    SCIFI,
}

impl Genre{
    fn get_text(&self) -> &str{
        match &self {
            Genre::ACTION => "Action",
            Genre::ADVENTURE => "Adventure",
            Genre::COMEDY => "Comedy",
            Genre::SCIFI => "Sci-Fi",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Details {
    movie: Movie,
    description: String,
    casts: Vec<String>,
    directors: Vec<String>,
    genres: Vec<String>,
    duration: String,
    release_date: String,
    id:String,
    cookie:String,
}


impl Parser {
    pub fn get_page_movies(&self, html: String) -> Result<Vec<Movie>, Box<dyn Error>> {
        let document = Html::parse_document(html.as_str());
        let container = Selector::parse("#xbrd > div.mxwd > div.dflex")?;
        let movie_container = Selector::parse("div > a")?;
        match document.select(&container).next() {
            None => Err("Unable to find the movie container".into()),
            Some(element) => {
                let img_selector = Selector::parse("img")?;
                let year_selector = Selector::parse(".hd.hdy")?;
                let quality_selector = Selector::parse(".hd.hda")?;
                let mut movies: Vec<Movie> = vec![];
                for movie_div in element.select(&movie_container) {
                    let url = movie_div.value().attr("href");
                    let title = movie_div.value().attr("title");
                    let image = movie_div.select(&img_selector).next().ok_or("Unable to get cover image".to_string())?.value().attr("data-src");
                    let year = movie_div.select(&year_selector).next().ok_or("Unable to get release date".to_string())?.text().collect::<String>();
                    let quality = movie_div.select(&quality_selector).next().ok_or("Unable to get quality".to_string())?.text().collect::<String>();
                    if url.is_some() && image.is_some() && title.is_some() {
                        movies.push(Movie { title: title.unwrap().parse()?, img: format!("https:{}", image.unwrap().parse::<String>()?), url: url.unwrap().parse()?, quality, year })
                    }
                }
                return Ok(movies);
            }
        }
    }
    pub fn get_woolty_url(&self, html:String) -> Result<String, Box<dyn Error>>{
        let document = Html::parse_document(html.as_str());
        let s1 = &Selector::parse("iframe")?;
        if let Some(url) = document.select(s1).next().ok_or("unable to get details".to_string())?.value().attr("src") {
            Ok(url.to_string())
        }else{
            Err("Unable to get url".into())
        }
    }
    pub fn get_movie_details(&self, html: String, cookie: String, movie: Movie) -> Result<Details, Box<dyn Error>>{
        let document = Html::parse_document(html.as_str());
        let s1 = &Selector::parse("#alger > div > div.marl > div.date")?;
        let s2 = &Selector::parse("#alger > div > div.marl > div.fimm")?;
        let s3 = &Selector::parse("#shd")?;
        let e1 = document.select(s1).next().ok_or("unable to get details".to_string())?;
        let e2 = document.select(s2).next().ok_or("unable to get details".to_string())?;
        let e3 = document.select(s3).next().ok_or("unable to get details".to_string())?;
        if let Some((duration, genres, release_date)) = Self::split_input(e1.text().collect::<String>().as_str()){
            let paragraph = Selector::parse("p")?;
            let data = e2.select(&paragraph).map(|e|  {
                e.text().collect::<String>().replace("Director: ", "").replace("Cast: ", "").split(", ").map(|t| t.to_string()).collect::<Vec<_>>()
            }).collect::<Vec<_>>();
            let regex = Regex::new(r#"_3chk.+?\);"#)?;
            let cookie= if let Some(captured) = regex.captures_iter(html.as_str()).last() {
                let input_str = captured.get(0).unwrap().as_str();
                let regex = Regex::new(r#"'(\w+)',\s*'(\w+)'"#).unwrap();
                if let Some(captures) = regex.captures(input_str) {
                    let first_capture = captures.get(1).unwrap().as_str();
                    let second_capture = captures.get(2).unwrap().as_str();
                    format!("{}; {}={};",cookie, first_capture, second_capture)
                }else{
                    cookie
                }
            }else{
                cookie
            };
            let id = e3.value().attr("data-ins").unwrap().to_string();
            if data.len() == 3{
                return Ok(Details{id, movie, genres, duration, release_date, cookie, description: data[0][0].clone(), directors: data[1].clone(), casts: data[2].clone()})
            }
        }
        Err("Unknown to get all movie details".into())
    }
    fn split_input(input: &str) -> Option<(String, Vec<String>, String)> {
        let parts: Vec<&str> = input.split(" | ").collect();
        if parts.len() >= 3 {
            let duration = parts[0].to_string();
            let genres = parts[1].to_string().split(", ").map(|c| c.to_string()).collect::<Vec<_>>();
            let release_date = parts[2].to_string();
            Some((duration, genres, release_date))
        } else {
            None
        }
    }
}

pub trait GoojaraImpl {
    fn search(&mut self, title: String) -> Result<Vec<Movie>, Box<dyn Error>>;
    fn details(&mut self, movie: Movie) -> Result<Details, Box<dyn Error>>;
    fn get_genre_movies(&mut self, genre: Genre, page:usize) -> Result<Vec<Movie>, Box<dyn Error>>;
    fn get_cookie_data(&mut self, url: &str, id:&str, old_cookie:&str) -> Result<String,  Box<dyn Error>>;
    fn parse_cookie(&mut self, cookie_list:List) -> Result<String,  Box<dyn Error>>;
    fn generate_hash_cookie(&mut self, text: &str, id:&str) -> Result<String, Box<dyn Error>>;
    fn woolty(&mut self, details: Details) -> Result<Wootly, Box<dyn Error>>;
}

impl  GoojaraImpl for Goojara {
    fn search(&mut self, title: String) -> Result<Vec<Movie>, Box<dyn Error>> {
        todo!()
    }

    fn details(&mut self, movie: Movie) -> Result<Details, Box<dyn Error>> {
        let mut request =  Request::new(movie.url.clone(), None, HttpMethod::GET, None);
        match request.execute(self.cookies.as_str()) {
            Ok(_) => {
                match request.get_response() {
                    Ok(data) => {
                        let html = data.get_text()?;
                        let mut details = self.parser.get_movie_details(html, self.cookies.clone(), movie)?;
                        match self.get_cookie_data(details.movie.url.as_str(), details.id.as_str(), details.cookie.as_str()) {
                            Ok(cookie) => {
                                details.cookie.push_str(format!(" {}", cookie).as_str())
                            }
                            Err(_) => {}
                        }
                        Ok(details)
                    }
                    Err(e) => {
                        error!("{}", e);
                        Err("Failed to read response from the host".into())
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                Err("Failed to execute request for the movie".into())
            }
        }
    }

    fn get_genre_movies(&mut self, genre: Genre, page:usize) -> Result<Vec<Movie>, Box<dyn Error>> {
        let request = &mut Request::new(format!("https://www.goojara.to/watch-movies-genre-{}?p={}", genre.get_text(), page), None, HttpMethod::GET, None);
        match request.execute(self.cookies.as_str()) {
            Ok(_) => {
                match request.get_response() {
                    Ok(data) => {
                        let html = data.get_text()?;
                        match request.get_cookie() {
                            Ok(cookie) => {
                               if let Ok(cookie) = self.parse_cookie(cookie){
                                   self.cookies = cookie
                               }
                            }
                            Err(_) => {}
                        }
                        return self.parser.get_page_movies(html);
                    }
                    Err(e) => {
                        error!("{}", e);
                        Err("Failed to read response from the host".into())
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                Err("Failed to execute request for the movie".into())
            }
        }
    }

    fn get_cookie_data(&mut self, url: &str, id:&str, old_cookie:&str) -> Result<String, Box<dyn Error>> {
        let headers = vec![format!("cookie: {}", old_cookie), "Content-type: application/x-www-form-urlencoded".to_string()];
        let request = &mut Request::new(format!("{}?p=2", url), Some(headers), HttpMethod::POST, Some("act=1".to_string().into_bytes()));
        match request.execute(old_cookie) {
            Ok(_) => {
                match request.get_response() {
                    Ok(data) => {
                        let html = data.get_text()?;
                        return self.generate_hash_cookie(html.as_str(), id);
                    }
                    Err(e) => {
                        error!("{}", e);
                        Err("Failed to read response from the host".into())
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
                Err("Failed to execute request for the movie".into())
            }
        }
    }

    fn parse_cookie(&mut self, cookie_list: List) -> Result<String, Box<dyn Error>> {
        match cookie_list.iter().last() {
            None => Err("No cookie found".into()),
            Some(bytes) => {
                let cookie_string = String::from_utf8_lossy(bytes).to_string();
                let parse_cookie = cookie_string.split("0\t").last().ok_or("unable to parse cookie")?.replace("\t", "=");
                Ok(parse_cookie)
            }
        }
    }

    fn generate_hash_cookie(&mut self, text: &str, id: &str) -> Result<String, Box<dyn Error>> {
        let text_parts: Vec<&str> = text.split(" ").collect();
        let b = &id[id.len() - 4..];
        let c = format!("{}{}", &id[7..10], b);
        let d = &id[id.len() - 2..];
        let g = text_parts.get(d[1..2].parse::<usize>()?).ok_or("Index out of range")?;
        let mut h = String::from("_");
        let f = text_parts.get(d[0..1].parse::<usize>().unwrap()).unwrap();
        for j in b.chars() {
            let index = j.to_digit(10).ok_or("Failed to parse character to digit")? as usize;
            let lower_f = f.to_lowercase();
            let nth_char = lower_f.chars().nth(index).ok_or("Index out of range")?;
            h.push(nth_char);
        }
        let mut code = String::from("");
        for i in c.chars() {
            let position = i.to_digit(10).ok_or("Failed to parse character to digit")? as usize;
            let char_at_pos = g.chars().nth(position).ok_or("Index out of range")?;
            code.push(char_at_pos);
        }
        Ok(format!("{}={};", h, SimpleHash::hash(code.as_str()).to_uppercase()))
    }

    fn woolty(&mut self, details: Details) -> Result<Wootly, Box<dyn Error>> {
        let movie_id = details.movie.url.split("/").last().ok_or("unable to get movie id".to_string())?.to_string();
        let timestamp = (Local::now() +chrono::Duration::minutes(1)).timestamp();
        let sig = SimpleHash::hash(
            format!("{}{}{}", movie_id, timestamp,details.id).as_str()
        );
        let hash = SimpleHash::hash(
            format!("{}{}", movie_id,details.id).as_str()
        );
        let url = format!("{}?p=2&sig={}&exp={}", details.movie.url, sig, timestamp);
        let body = format!("act=2&ogn={}&hash={}", movie_id, hash);
        let headers = vec![format!("cookie: {}", details.cookie),
                           "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/118.0".to_string(),
                           "Accept: */*".to_string(),
                           "Accept-Language: en-US,en;q=0.5".to_string(),
                           "Content-type: application/x-www-form-urlencoded".to_string(),
                           "Alt-Used: ww1.goojara.to".to_string(),
                           "Sec-Fetch-Dest: empty".to_string(),
                           "Sec-Fetch-Mode: cors".to_string(),
                           "Sec-Fetch-Site: same-origin".to_string(),
        ];
        let request = &mut Request::new(url, Some(headers), HttpMethod::GET, Some(body.into_bytes()));
        // let request = &mut Request::new(url, Some(headers), HttpMethod::GET, None);
        match request.execute(details.cookie.as_str()) {
            Ok(()) => {
                match request.get_response() {
                    Ok(data) => {
                        let html = data.get_text()?;
                        let url = self.parser.get_woolty_url(html)?;
                        Ok(Wootly::new(url.as_str()))
                    }
                    Err(e) => {
                        return Err(format!("error reading response:\n{}", e).into())
                    }
                }
            }
            Err(e) => {
                return Err("Unable to send woolty request".into())
            }
        }
    }
}

impl  Goojara {
    pub fn new() -> Self {
        Self {
            parser: Parser {},
            cookies: String::new()
        }
    }
}