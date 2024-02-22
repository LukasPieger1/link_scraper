use crate::error::ExtractionError;
use crate::parser::{find_urls, UrlContainer};
use reqwest::blocking::Response;
use reqwest::{blocking, Url};

pub fn get(url: Url) -> Result<Response, ExtractionError> {
    let res: Response = blocking::get(url)?;

    Ok(res)
}

impl UrlContainer for Response {
    fn extract_urls(self) -> Result<Vec<Url>, ExtractionError> {
        let plain_text = self.text()?;
        Ok(find_urls(&plain_text))
    }
}

impl From<reqwest::Error> for ExtractionError {
    fn from(value: reqwest::Error) -> Self {
        ExtractionError::new(Some("during request to website"), Some(Box::new(value)))
    }
}
impl From<std::io::Error> for ExtractionError {
    fn from(value: std::io::Error) -> Self {
        ExtractionError::new(Some("During parsing "), Some(Box::new(value)))
    }
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
        let urls = site_content.extract_urls();
        println!("Found URLs: {:?}", urls)
    }
}
