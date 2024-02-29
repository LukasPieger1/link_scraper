use lazy_static::lazy_static;
use linkify::LinkFinder;
use reqwest::{Url};

lazy_static! {
    static ref FINDER: LinkFinder = LinkFinder::new();
}

//TODO further formats: docx, doc, xlsx, powerpoint, rtf, json, xml, tsv, csv

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::link_extractor::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
pub fn find_urls(content: &str) -> Vec<&str> {
    FINDER.links(content).map(|link| link.as_str()).collect()
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