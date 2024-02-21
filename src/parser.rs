use std::fmt::{Debug, Display, Formatter};
use reqwest::{blocking, Url};
use regex::{Regex};
use log::{trace};
use thiserror::Error;

#[derive(Error, Debug)]
pub struct ExtractionError {
    msg: String,
    source: Option<Box<dyn std::error::Error>>, // TODO what am I doing wrong here? I think I can't make this an enum because my other mods want to expand what an ExtractionError can be.
}

impl ExtractionError {
    pub fn new(msg: Option<&str>, source: Option<Box<dyn std::error::Error>>) -> Self {
        let message: String = {
            if let Some(text) = msg { text.to_string() }
            else if let Some(err) = &source { format!("Raised by {}", err) }
            else { "No further information available.".to_string() }
        };
        Self {
            msg: message,
            source
        }
    }
}

impl Display for ExtractionError {
    // TODO I think I'm doing something wrong here ...do I actually need to implement Display with thiserror?
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(src) = &self.source {
            write!(f, "ExtractionError: {}; Source: {}", self.msg, src)
        } else {
            write!(f, "ExtractionError: {}", self.msg)
        }
    }
}

trait UrlContainer {
    fn extract_urls(&self) -> Result<Vec<Url>, ExtractionError>;
}

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::parser::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
pub fn find_urls(content: &str) -> Vec<&str> {
    let url_regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}").unwrap();
    let mut results:Vec<&str> = vec![];
    let matches = url_regex.captures_iter(content);

    for one_match in matches {
        let url = one_match.get(0).unwrap().as_str();
        trace! ("Found an URL! {url}");
        results.push(url);
    }

    results
}