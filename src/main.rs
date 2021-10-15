extern crate regex;
extern crate reqwest;
extern crate select;

use regex::Regex;
use select::document::Document;
use select::predicate::Attr;
use std::{env, fs, io, vec};

struct ImageItem {
    data_id: String,
    data_type: String,
    data_server: String,
}

fn main() {
    //get the url from command line arguments
    let args: Vec<String> = env::args().collect();
    let input_url = match args.len() {
        1 => panic!("Please specify a valid url."),
        _ => &args[1],
    };
    //get base url. This does nothing for now but will eventually extract the correct url from the different variations
    let base_url = get_base_url(input_url);

    //get parseable document from the given base_url
    let html_document = match get_html_from_url(&base_url) {
        Ok(page) => match page {
            Ok(page) => page,
            Err(e) => panic!("Unable to continue. Could not download the HTML {:?}", e),
        },
        Err(e) => panic!("Unable to continue. Could not parse the HTML {:?}", e),
    };
    //get the total number of pages of wallpapeprs
    let total_pages = find_total_pages(&html_document);

    for page in 1..=match total_pages {
        Ok(total_pages) => total_pages,
        Err(e) => panic!("Cannot parse the page count as integer {:?}", e),
    } {
        let page_url = base_url.to_string() + "&page=" + &page.to_string();

        let current_page = match get_html_from_url(&page_url) {
            Ok(item) => match item {
                Ok(item) => item,
                Err(e) => {
                    eprintln!("Skipped page {}; Unable to download page: {:?}", page, e);
                    continue;
                }
            },
            Err(e) => {
                eprintln!("Skipped page {}; Unable to parse page HTML: {:?}", page, e);
                continue;
            }
        };
        for item in get_all_page_items(&current_page) {
            get_image(&item);
        }
    }

    //println!("{}", html_text);
}

fn get_html_from_url(url: &str) -> Result<Result<Document, io::Error>, reqwest::Error> {
    match reqwest::blocking::get(url) {
        Ok(page) => Ok(Document::from_read(page)),
        Err(e) => Err(e),
    }
}

fn find_total_pages(html: &Document) -> Result<u32, std::num::ParseIntError> {
    let html_span = match html.find(Attr("class", "btn btn-info btn-lg")).next() {
        Some(html_span) => html_span.text(),
        None => panic!("Unable to parse the page count (html error). Cannot continue."),
    };
    let page_count = match Regex::new(r"/(?P<cnt>\d+)").unwrap().captures(&html_span) {
        Some(page_count) => match page_count.name("cnt") {
            Some(page_count) => page_count.as_str(),
            None => panic!("Unable to parse the page count (regex failure). Cannot continue."),
        },
        None => panic!("Unable to parse the page count (regex failure). Cannot continue."),
    };
    return page_count.parse::<u32>();
}

fn get_all_page_items(html: &Document) -> Vec<ImageItem> {
    let mut result = vec![];
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
    let image_download_url = String::from("https://initiate.alphacoders.com/download/wallpaper/")
        + &item.data_id
        + "/"
        + &item.data_server
        + "/"
        + &item.data_type;
    let image_output_filename = String::from(&item.data_id) + "." + &item.data_type;
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

fn get_base_url(url: &str) -> &str {
    // TODO: parse other types of input urls correctly:
    url
}

//TODO: mkdir for the title of the album and download into there
//fn get_title
