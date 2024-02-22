use lazy_static::lazy_static;
use reqwest::{Url};
use regex::{Regex};
use log::{trace};
use crate::error::ExtractionError;


lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}").unwrap();
}

pub trait UrlContainer {
    fn extract_urls(self) -> Result<Vec<Url>, ExtractionError>;
}

//TODO further formats: docx, doc, xlsx, powerpoint, rtf, json, xml, tsv, csv

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::parser::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
fn find_url_strings(content: &str) -> Vec<&str> {
    let mut results:Vec<&str> = vec![];
    let matches = URL_REGEX.captures_iter(content);

    for one_match in matches {
        let url = one_match.get(0).unwrap().as_str();
        trace! ("Found an URL! {url}");
        results.push(url);
    }

    results
}

pub fn find_urls(content: &str) -> Vec<Url>{
    find_url_strings(content).iter()
        .filter_map(|res| parse(res))
        .collect()
}

pub fn parse(url_string: &str) -> Option<Url> {
    let res = Url::parse(url_string);
    match res {
        Ok(url) => Some(url),
        Err(err) => {
            println!("Unable to parse URL {}", err);
            None
        }
    }
}