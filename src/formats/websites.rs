use crate::link_extractor::find_urls;
use reqwest::blocking::Response;
use reqwest::{blocking, Url};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebsiteExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

pub fn get(url: Url) -> Result<Response, WebsiteExtractionError> {
    let res: Response = blocking::get(url)?;
    Ok(res)
}

#[cfg(feature = "link_extraction")]
fn extract_links(response: Response) -> Result<Vec<String>, WebsiteExtractionError> {
    let plain_text = response.text()?;
    Ok(find_urls(&plain_text)
        .iter()
        .map(|it| it.to_string())
        .collect())
}

#[cfg(feature = "link_extraction")]
pub fn extract_links_from_url(url: Url) -> Result<Vec<String>, WebsiteExtractionError> {
    extract_links(get(url)?)
}

#[cfg(feature = "link_extraction")]
pub fn extract_links_from_html(html_byte_array: &str) -> Vec<String> {
    // TODO do you want this to be a &[u8] instead?
    find_urls(html_byte_array)
        .iter()
        .map(|it| it.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref TEST_URL: String = "https://github.com/llvm/llvm-project/issues/55760".to_string();
            // static ref TEST_URL: String = "https://www.google.com".to_string();
    }

    #[test]
    fn get_some_website() {
        let url = Url::parse(&TEST_URL).unwrap();
        let result = get(url);
        result.unwrap();
    }

    #[test]
    fn find_urls_in_website() {
        let url = Url::parse(&TEST_URL).unwrap();
        let site_content = get(url).unwrap();
        let urls = extract_links(site_content);
        println!("Found URLs: {:?}", urls)
    }
}
