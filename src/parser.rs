use reqwest::{blocking, Error, Url};
use regex::{Regex};
use crate::parser::MyError::{RequestError, StdIoError};
use log::{trace};

#[derive(Debug)] //TODO why do I need this here?
pub enum MyError {
    RequestError(reqwest::Error),
    StdIoError(std::io::Error)
}
impl From<reqwest::Error> for MyError {
    fn from(value: Error) -> Self {
        RequestError(value)
    }
}
impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        StdIoError(value)
    }
}

pub fn get(url: Url) -> Result<String, MyError> {
    let res = blocking::get(url)?;
    let body = res.text()?;

    Ok(body)
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use super::*;

    #[test]
    fn get_some_website() {
        let url = Url::parse("https://github.com/llvm/llvm-project/issues/55760").unwrap();
        // let url = Url::parse("https://www.google.com").unwrap();
        let result = get(url);
        match result {
            Ok(result_as_string) => { println!("{}", result_as_string) }
            Err(RequestError(err)) => {
                err
            }
            Err(StdIoError(err)) => {

            }
            Err(my_error) => {
                if let RequestError(err) = my_error { panic!("Request no worky :( {:?}", err); }
                else { panic!("Couldn't parse :(") }
            }
        };

        result.err().unwrap();
    }

    #[test]
    fn find_urls_in_website() {
        use itertools::Itertools;

        // let url = Url::parse("https://github.com/llvm/llvm-project/issues/55760").unwrap();
        let url = Url::parse("https://www.google.com").unwrap();
        let urls = get(url).map(
            |value| find_urls(&value).into_iter().unique().collect::<Vec<_>>()
        ).expect("There should be something");
        println!("Found URLs: {:?}", urls)
    }
}