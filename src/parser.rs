use std::io::Read;
use reqwest::{blocking, Error, Url};
use crate::parser::MyError::{RequestError, StdIoError};

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
    let mut res = blocking::get(url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let url = Url::parse("https://github.com/llvm/llvm-project/issues/55760").unwrap();
        // let url = Url::parse("https://www.google.com").unwrap(); // TODO: Please explain to me why this doesn't work :D
        let result = get(url);
        match  { result } {
            Ok(result_as_string) => { println!("{}", result_as_string) }
            Err(my_error) => {
                if let RequestError(err) = my_error { println!("Request no worky :( {:?}", err); }
                else { println!("Couldn't parse :(") }
            }
        };

    }
}