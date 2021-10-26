extern crate regex;
extern crate select;
extern crate ureq;

use anyhow::Context;
use anyhow::Result;
use argh::FromArgs;
use regex::Regex;
use select::document::Document;
use select::predicate::Attr;
use std::{fs, io, path::Path};

struct ImageItem {
    data_id: String,
    data_type: String,
    data_server: String,
}

//Argument parser
#[derive(FromArgs)]
#[argh(
    description = "Downloads the Alphacoders gallery provided by the given url. Currently only supports \"desktop wallpapers\" category."
)]
struct ParsedArgs {
    #[argh(positional)]
    url: String,
}

fn main() -> Result<()> {
    //parse the arguments
    let parsed_args: ParsedArgs = argh::from_env();
    //get base url. This does nothing for now but will eventually extract the correct url from the different variations
    let base_url = get_base_url(parsed_args.url)?;

    //get parseable document from the given base_url
    let html_document = get_html_from_url(&base_url)?;
    //get the total number of pages of wallpapeprs
    let total_pages =
        find_total_pages(&html_document).context("Could not parse page count as integer")?;

    //create the output directory if it doesn't exist.
    let album_title = get_album_title(&html_document)?;
    let output_dir = Path::new(&album_title);
    fs::create_dir_all(output_dir)?;

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
            match get_image(&item, &output_dir) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Skipped an image due to error: {:?}", e);
                    continue;
                }
            };
        }
    }

    Ok(())
}

fn get_html_from_url(url: &str) -> Result<Document> {
    Ok(Document::from_read(ureq::get(url).call()?.into_reader())?)
}

fn find_total_pages(html: &Document) -> Result<u32> {
    let html_span = html
        .find(Attr("class", "btn btn-info btn-lg"))
        .next()
        .context("Unable to parse the page count (html error). Cannot continue.")?
        .text();
    let page_count = Regex::new(r"/(?P<cnt>\d+)")
        .unwrap()
        .captures(&html_span)
        .context("Unable to parse the page count (regex failure)")?
        .name("cnt")
        .context("Unable to parse the page count (regex failure)")?
        .as_str();
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

fn get_image(item: &ImageItem, output_dir: &Path) -> Result<()> {
    let image_download_url = format!(
        "https://initiate.alphacoders.com/download/wallpaper/{}/{}/{}",
        &item.data_id, &item.data_server, &item.data_type
    );

    let image_output_filename =
        output_dir.join(Path::new(&format!("{}.{}", &item.data_id, &item.data_type)));
    println!(
        "Downloading image to {}",
        image_output_filename.to_string_lossy()
    );
    let mut outfile = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(image_output_filename)?;

    let mut web_response = ureq::get(&image_download_url).call()?.into_reader();
    io::copy(&mut web_response, &mut outfile)
        .context("Failed to write image content into file.")?;
    Ok(())
}

fn get_base_url(url: String) -> Result<String> {
    // Deals with a potentially missing "?" so that we can append &page to the url.
    if !Regex::new(r"\?").unwrap().is_match(&url) {
        return Ok(url + "?");
    }
    Ok(url)
}

fn get_album_title(html: &Document) -> Result<String> {
    Ok(html
        .find(Attr("class", "title"))
        .nth(0)
        .context("Could not determine the title of the album")?
        .text()
        .trim()
        .to_string())
}
