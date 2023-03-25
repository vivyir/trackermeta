//! This is a library crate for working with the [Modarchive](https://modarchive.org)
//! website, it is very barebones and simple to work with, please check out the
//! documentation for [`ModInfo`] and its methods for more info, do be sure to look
//! at the examples aswell!
//!
//! (This is the Reborn update, v0.5.x)
//!
//! ## Example: Get module info as a struct using a module id
//! ```rust
//! use trackermeta::ModInfo;
//!
//! fn main() {
//!     let modinfo = ModInfo::get(51772).unwrap();
//!     println!("{:#?}", modinfo);
//! }
//! ```
//!
//! ## Example: Resolve filename to id then use id to get the info as struct
//! ```rust
//! use trackermeta::ModInfo;
//!
//! fn main() {
//!     let modid = ModInfo::resolve_filename("noway.s3m").unwrap()[0].id;
//!     let modinfo = ModInfo::get(modid).unwrap();
//!     println!("{:#?}", modinfo);
//! }
//! ```
//!
//! There are more examples other than these which showcase more, remember
//! to check the `examples` directory!
//!
//! [Modarchive]: https://modarchive.org
#![allow(clippy::needless_doctest_main)]
#![forbid(unsafe_code)]

use chrono::prelude::{DateTime, Utc};

// https://stackoverflow.com/a/64148190
fn iso8601_time(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = (*st).into();
    format!("{}", dt.format("%+"))
}

/// Error enum for functions in the crate that return a [`Result`]
#[derive(Debug)]
pub enum Error {
    NotFound,
}

/// Simple struct to represent a search result, id and filename will be provided in each
#[derive(Debug)]
pub struct ModSearch {
    pub id: u32,
    pub filename: String,
}

/// Struct containing all of the info about a module
#[derive(Debug)]
pub struct ModInfo {
    /// The module ID of the module on modarchive
    pub id: u32,
    /// The filename of the module
    pub filename: String,
    /// The title of the module
    pub title: String,
    /// The file size of the module, use the
    /// crate `byte-unit` to convert them to other units
    pub size: String,
    /// The MD5 hash of the module file as a string
    pub md5: String,
    /// The format of the module, for example `XM`, `IT`
    /// or `MOD` and more, basically the extension of the
    /// module file
    pub format: String,
    /// Spotlit module or not
    pub spotlit: bool,
    /// Download count of the module at the time of scraping
    pub download_count: u32,
    /// Times the module has been favourited at the time of
    /// scraping
    pub fav_count: u32,
    /// The time when it was scraped
    pub scrape_time: String,
    /// The channel count of the module
    pub channel_count: u32,
    /// The genre of the module
    pub genre: String,
    /// The upload date of the module
    pub upload_date: String,
    /// The instrument text of the module
    pub instrument_text: String,
}

cfg_if::cfg_if! {
    if #[cfg(feature = "infinity-retry")] {
        fn inner_request(mod_id: u32) -> String{
            loop {
                match ureq::get(
                    format!(
                        "https://modarchive.org/index.php?request=view_by_moduleid&query={}",
                        mod_id
                    )
                    .as_str(),
                )
                .timeout(std::time::Duration::from_secs(60))
                .call() {
                    Ok(req) => {
                        return req.into_string().unwrap()
                    }
                    Err(_) => continue,
                };
            }
        }
    } else {
        fn inner_request(mod_id: u32) -> String {
            let body = ureq::get(
                format!(
                    "https://modarchive.org/index.php?request=view_by_moduleid&query={}",
                    mod_id
                )
                .as_str(),
            )
            .timeout(std::time::Duration::from_secs(60))
            .call()
            .unwrap()
            .into_string()
            .unwrap();

            body
        }
    }
}

impl ModInfo {
    /// Probably the singular most important function in this crate, takes a mod id (can be
    /// generated at random, deliberately entered or acquired by resolving a filename and
    /// picking a search result), and then gives you a full [`ModInfo`] struct.
    pub fn get(mod_id: u32) -> Result<ModInfo, crate::Error> {
        let body = inner_request(mod_id);

        let dom = tl::parse(body.as_ref(), tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let id = mod_id;
        let scrape_time = iso8601_time(&std::time::SystemTime::now());

        let valid = dom.get_elements_by_class_name("mod-page-archive-info")
                .next()
                .is_some();

        if !valid {
            return Err(crate::Error::NotFound);
        }

        let filename = {
            dom.get_elements_by_class_name("module-sub-header")
                .next()
                .unwrap() // we can unwrap because if its absent we've already errored up above
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace('(', "")
                .replace(')', "")
        };

        let title = {
            escaper::decode_html(
                &dom.query_selector("h1")
                    .and_then(|mut iter| iter.next())
                    .unwrap()
                    .get(parser)
                    .unwrap()
                    .inner_text(parser)
                    .replace(&format!(" ({})", &filename), ""),
            )
            .unwrap()
        };

        let size = {
            dom.query_selector("li.stats")
                // get the 8th hit (nth starts from 0)
                .and_then(|mut iter| iter.nth(7))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("Uncompressed Size: ", "")
        };

        let md5 = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.nth(4))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("MD5: ", "")
        };

        let format = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.nth(5))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("Format: ", "")
        };

        let spotlit = dom.get_elements_by_class_name("mod-page-featured")
                .next()
                .is_some();

        let download_count = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.nth(2))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("Downloads: ", "")
                .parse()
                .unwrap()
        };

        let fav_count = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.nth(3))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("Favourited: ", "")
                .replace(" times", "")
                .parse()
                .unwrap()
        };

        let channel_count = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.nth(6))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("Channels: ", "")
                .parse()
                .unwrap()
        };

        dom.query_selector("a.master");

        let genre = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.nth(8))
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .replace("Genre: ", "")
        };

        let upload_date = {
            dom.query_selector("li.stats")
                .and_then(|mut iter| iter.next())
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser)
                .split(" times since ")
                .nth(1)
                .unwrap()
                .replace(" :D", "")
                .trim()
                .into()
        };

        let instrument_text = {
            escaper::decode_html(
                &dom.query_selector("pre")
                    .and_then(|mut iter| iter.nth(1))
                    .unwrap()
                    .get(parser)
                    .unwrap()
                    .inner_text(parser),
            )
            .unwrap()
            .trim()
            .into()
        };

        Ok(ModInfo {
            id,
            filename,
            title,
            size,
            md5,
            format,
            spotlit,
            download_count,
            fav_count,
            scrape_time,
            channel_count,
            genre,
            upload_date,
            instrument_text,
        })
    }

    /// Returns a modarchive download link for the given module, you can get this struct by using
    /// [`ModInfo::get()`], or search using [`ModInfo::resolve_filename()`], if you're using the
    /// resolver function please consider using the [`ModSearch::get_download_link()`] method
    /// instead.
    pub fn get_download_link(&self) -> String {
        format!(
            "https://api.modarchive.org/downloads.php?moduleid={}#{}",
            self.id, self.filename
        )
    }

    /// Searches for your string on Modarchive and returns the results on the first page (a.k.a
    /// only up to the first 40) as a vector of [`ModSearch`]
    pub fn resolve_filename(filename: &str) -> Result<Vec<ModSearch>, crate::Error> {
        let body: String = ureq::get(
                format!(
                    "https://modarchive.org/index.php?request=search&query={}&submit=Find&search_type=filename",
                    filename
                )
                .as_str(),
            )
            .call()
            .unwrap()
            .into_string()
            .unwrap();

        let dom = tl::parse(&body, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let status = dom.query_selector("h1.site-wide-page-head-title");

        match status {
            Some(_) => {}
            None => return Err(crate::Error::NotFound),
        };
        // from this point on we can unwrap after each query selector
        // because our info will for sure be present.

        let links: Vec<ModSearch> = dom
            .query_selector("a.standard-link[title]")
            .unwrap()
            .map(|nodehandle| {
                let node = nodehandle.get(parser).unwrap();

                let id = match node.as_tag().unwrap().attributes().get("href") {
                    Some(Some(a)) => a
                        .as_utf8_str()
                        .split("query=")
                        .nth(1)
                        .unwrap()
                        .parse()
                        .unwrap(),
                    Some(None) => unreachable!(),
                    None => unreachable!(),
                };

                let filename = node.inner_text(parser).into();

                ModSearch { id, filename }
            })
            .collect();

        Ok(links)
    }
}

impl ModSearch {
    /// Get the download link of this specific module.
    pub fn get_download_link(&self) -> String {
        format!(
            "https://api.modarchive.org/downloads.php?moduleid={}#{}",
            self.id, self.filename
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ModInfo;

    #[test]
    fn instr_text() {
        let instr_text = ModInfo::get(61772).unwrap().instrument_text;
        assert_eq!(
            instr_text,
            "7th  Dance

             By:
 Jari Ylamaki aka Yrde
  27.11.2000 HELSINKI

            Finland
           SITE :
  www.mp3.com/Yrde"
        );
    }

    #[test]
    fn invalid_modid() {
        let invalid = ModInfo::get(30638);
        assert!(invalid.is_err());
    }

    #[test]
    fn valid_modid() {
        let valid = ModInfo::get(99356);
        assert!(valid.is_ok());
    }

    #[test]
    fn spotlit_modid() {
        let module = ModInfo::get(158263).unwrap();
        assert_eq!(module.spotlit, true);
    }

    #[test]
    fn name_resolving() {
        let mod_search = ModInfo::resolve_filename("virtual-monotone.mod");
        let mod_search = &mod_search.unwrap()[0];
        assert_eq!(mod_search.id, 88676);
        assert_eq!(
            mod_search.get_download_link().as_str(),
            "https://api.modarchive.org/downloads.php?moduleid=88676#virtual-monotone.mod"
        );
    }

    #[test]
    fn dl_link_modinfo() {
        let modinfo = ModInfo::get(41070).unwrap();
        assert_eq!(
            modinfo.get_download_link().as_str(),
            "https://api.modarchive.org/downloads.php?moduleid=41070#fading_horizont.mod"
        );
    }
}
