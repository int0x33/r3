extern crate termion;
extern crate reqwest;
extern crate select;

use std::fs::File;
use std::io::{BufReader, BufRead};
use std::io::Error;
use std::result::Result;

use select::document::Document;
use select::predicate::Name;

fn get_tcp(url: &str) -> Result<(), Error>
{
    let url2req = format!("http://{}", url);
    let response = reqwest::get(&url2req).expect("Failed to send request");
    if response.status() == 200 {
        //println!("Status: {}", response.status());
        //println!("Headers:\n{:#?}", response.headers());
        //let json = response.text();
       // println!("{:?}", json);
        Document::from_read(response)?
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .for_each(|x| println!("{}", x));
    } else {
        println!("FAILED");
    }
    Ok(())
}

fn run() -> Result<(), Error> {
    let path = "target/urls.txt";

    let input = File::open(path)?;
    let buffered = BufReader::new(input);

    for line in buffered.lines() {
        let url = String::from(line?);
        let _ = get_tcp(&url);
    }
    Ok(())
}

fn main() {
    let _ = run();
}