extern crate regex;
extern crate reqwest;
extern crate select;

use regex::Regex;
use select::document::Document;
use select::predicate::Attr;
use std::{env, fs, io};
use anyhow::Result;
use anyhow::Context;

struct ImageItem {
    data_id: String,
    data_type: String,
    data_server: String,
}

fn main() -> Result<()> {
    //get the url from command line arguments
    let input_url = env::args().nth(1).context("Please specify a valid url.")?;
    //get base url. This does nothing for now but will eventually extract the correct url from the different variations
    let base_url = get_base_url(input_url);

    //get parseable document from the given base_url
    let html_document = get_html_from_url(&base_url)?;
    //get the total number of pages of wallpapeprs
    let total_pages = find_total_pages(&html_document).context("Could not parse page count as integer")?;

    for page in 1..=total_pages {
        let page_url = format!("{}&page={}", base_url, page);
        
        //we currently skip a page if there is an error.
        let current_page = match get_html_from_url(&page_url) {
            Ok(item) => item,
            Err(e) => {
                eprintln!("Skipped page {} due to error: {:?}", page, e);
                continue;
            }
        };
        for item in get_all_page_items(&current_page) {
            get_image(&item);
        }
    }

    Ok(())
}

fn get_html_from_url(url: &str) -> Result<Document> {
    Ok(Document::from_read(reqwest::blocking::get(url)?)?)
}

fn find_total_pages(html: &Document) -> Result<u32> {
    let html_span = html.find(Attr("class", "btn btn-info btn-lg")).next().context("Unable to parse the page count (html error). Cannot continue.")?.text();
    let page_count = Regex::new(r"/(?P<cnt>\d+)").unwrap().captures(&html_span).context("Unable to parse the page count (regex failure)")?.name("cnt").context("Unable to parse the page count (regex failure)")?.as_str();
    Ok(page_count.parse::<u32>()?)
}

fn get_all_page_items(html: &Document) -> Vec<ImageItem> {
    let mut result = Vec::new();
    for item in html.find(Attr("class", "btn btn-primary btn-block download-button")) {
        result.push(ImageItem {
            data_id: match item.attr("data-id") {
                Some(item) => item.to_string(),
                None => continue,
            },
            data_type: match item.attr("data-type") {
                Some(item) => item.to_string(),
                None => continue,
            },
            data_server: match item.attr("data-server") {
                Some(item) => item.to_string(),
                None => continue,
            },
        });
    }
    result
}

fn get_image(item: &ImageItem) {
    let image_download_url = format!("https://initiate.alphacoders.com/download/wallpaper/{}/{}/{}", &item.data_id, &item.data_server, &item.data_type);
    let image_output_filename = format!("{}.{}", &item.data_id, &item.data_type);
    println!("Downloading image to {}", image_output_filename);
    //TODO how do we propagate these errors upwards so we only have to skip this particular item if there's an error?
    let mut w_resp = match reqwest::blocking::get(image_download_url) {
        Ok(image) => image,
        Err(e) => panic!("Unable to download image url {:?}", e),
    };
    let mut outfile = match fs::File::create(image_output_filename) {
        Ok(fd) => fd,
        Err(e) => panic!("Could not open output file {:?}", e),
    };
    io::copy(&mut w_resp, &mut outfile).expect("Failed to write image content into file.");
}

fn get_base_url(url: String) -> String {
    // TODO: parse other types of input urls correctly:
    url
}

//TODO: mkdir for the title of the album and download into there
//fn get_title
