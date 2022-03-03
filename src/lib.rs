//! This is a simple library and a small utility crate that helps with parsing
//! data from the website called [Modarchive], the
//! utility searches modarchive for the filename provided, gets the most likely
//! result, extracts module id and then gets the full details for it as a single
//! csv record which the structure of it can be seen in the docs of the
//! first function of the [`scraper::requests`] module or alternatively as a
//! [`scraper::ModInfo`] struct using the function
//! [`scraper::requests::get_full_details_as_struct`].
//!
//! ## Example: Get module info as a struct using a module id
//! ```rust
//! use trackermeta::scraper::ModInfo;
//!
//! fn main() {
//!     let modinfo = ModInfo::get(51772).unwrap();
//!     println!("{:#?}", modinfo);
//! }
//! ```
//!
//! ## Example: Resolve filename to id then use id to get the info as struct
//! ```rust
//! use trackermeta::scraper::{ModInfo, resolver};
//!
//! fn main() {
//!     let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
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

#[cfg(feature = "overridable")]
use platform_dirs::AppDirs;
#[cfg(feature = "overridable")]
use std::fs;
#[cfg(feature = "overridable")]
use std::io::{Read, Write};

// https://stackoverflow.com/a/64148190
fn iso8601_time(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
}

#[cfg(feature = "overridable")]
fn load_lines(
    mod_fl_line: usize,
    mod_inf_line: usize,
    mod_dl_line: usize,
) -> (usize, usize, usize) {
    let app_dirs = AppDirs::new(Some("trackermeta"), true).unwrap();
    let config_file_path = app_dirs.config_dir.join("line-overrides");

    if config_file_path.exists() {
        let mut file = fs::File::open(config_file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let contents = contents.trim().split(',');

        let loaded_mod_fl_line: usize = contents
            .clone()
            .next()
            .unwrap()
            .parse()
            .expect("Invalid digit for mod_filename_line in config");

        let loaded_mod_inf_line: usize = contents
            .clone()
            .nth(1)
            .unwrap()
            .parse()
            .expect("Invalid digit for mod_info_line in config");

        let loaded_mod_dl_line: usize = contents
            .clone()
            .nth(2)
            .unwrap()
            .parse()
            .expect("Invalid digit for mod_download_line in config");

        return (loaded_mod_fl_line, loaded_mod_inf_line, loaded_mod_dl_line);
    } else {
        fs::create_dir_all(&app_dirs.config_dir).unwrap();

        let mut file = fs::File::create(config_file_path).unwrap();
        file.write_all(format!("{},{},{}", mod_fl_line, mod_inf_line, mod_dl_line).as_bytes())
            .unwrap();

        return (mod_fl_line, mod_inf_line, mod_dl_line);
    };
}

/// The main module containing everything in the crate
pub mod scraper {
    #[cfg(feature = "overridable")]
    use crate::load_lines;

    use crate::iso8601_time;

    /// Error enum for functions in the crate that return a [`Result`]
    #[derive(Debug)]
    pub enum Error {
        NotFound,
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
        pub fn get(mod_id: u32) -> Result<ModInfo, crate::scraper::Error> {
            let body = inner_request(mod_id);

            let dom = tl::parse(body.as_ref(), tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();

            let id = mod_id;
            let scrape_time = iso8601_time(&std::time::SystemTime::now());

            let valid = {
                let mut iter = dom.get_elements_by_class_name("mod-page-archive-info");

                match iter.next() {
                    Some(_) => true,
                    None => false,
                }
            };

            if valid == false {
                return Err(crate::scraper::Error::NotFound);
            }

            let filename = {
                dom.get_elements_by_class_name("module-sub-header")
                    .next()
                    .unwrap() // we can unwrap because if its absent we've already errored up above
                    .get(parser)
                    .unwrap()
                    .inner_text(parser)
                    .replace("(", "")
                    .replace(")", "")
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

            let spotlit = {
                let mut iter = dom.get_elements_by_class_name("mod-page-featured");

                match iter.next() {
                    Some(_) => true,
                    None => false,
                }
            };

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
    }

    /// Get the instrument text/internal text by mod id
    ///
    /// This function gets the instrument text of any module id and
    /// returns a [`Result`] because the supplied module id may not
    /// be present, this function takes care of discarding comments
    /// and reviews if they exist and is the first fucntion to use
    /// a "semi-stable" anchor, since it detects comments and
    /// increments the anchor appropriately.
    ///
    /// Although even after all of that there is still a chance of
    /// failure since it hasn't gone thru rigorous testing, only
    /// some small modules here and there were tested, so don't
    /// treat it as a rock solid function.
    pub fn get_instrument_text(module_id: u32) -> Result<String, crate::scraper::Error> {
        let mut mod_status = "present";
        let mod_instr_text_div_line = 278;
        let mut mod_instr_text_line;
        let mut mod_instr_text = String::from("");

        let body: String = inner_request(module_id);

        let mod_status_text = body.split('\n').nth(184 - 1).unwrap();
        if mod_status_text.is_empty() {
            mod_status = "absent";
        }

        if mod_status != "absent" {
            {
                /* between these two comment lines is the only non-boilerplate code */
                let mut loopcounter = mod_instr_text_div_line;
                loop {
                    let local_text = body.split('\n').nth(loopcounter - 1).unwrap();

                    if local_text == "<div class=\"mod-page-instrument-text\">" {
                        break;
                    }

                    loopcounter += 1;
                }

                mod_instr_text_line = loopcounter + 9;

                let mod_spotlit_text = body.split('\n').nth(170 - 1).unwrap();
                if !mod_spotlit_text.is_empty() {
                    mod_instr_text_line += 6;
                }

                let mut loopcounter = mod_instr_text_line;
                loop {
                    let local_text = body.split('\n').nth(loopcounter - 1).unwrap();

                    if local_text == "</pre>" {
                        break;
                    }

                    mod_instr_text.push_str(local_text);
                    mod_instr_text.push('\n');

                    loopcounter += 1;
                }

                let mod_instr_text = escaper::decode_html(&mod_instr_text)
                    .unwrap()
                    .trim()
                    .to_string();
                /* ---------------------------------------------------------------- */

                Ok(mod_instr_text)
            }
        } else {
            Err(crate::scraper::Error::NotFound)
        }
    }

    /// Module containing scraper functions that resolve to a modarchive module ID
    pub mod resolver {
        /// Resolve a filename to a module id ([`u32`])
        ///
        /// Resolve a filename ([`&str`]) to a [`Result`] which on failure will return
        /// a variation of [`crate::scraper::Error`]. otherwise it will return a
        /// module id as u32 which you can use in functions of the [`crate::scraper::requests`]
        /// module that will ultimately give you all of the information about the module.
        pub fn resolve_mod_filename(mod_filename: &str) -> Result<u32, crate::scraper::Error> {
            let body: String = ureq::get(
                format!(
                    "https://modarchive.org/index.php?request=search&query={}&submit=Find&search_type=filename",
                    mod_filename
                )
                .as_str(),
            )
            .call()
            .unwrap()
            .into_string()
            .unwrap();

            let stat_line = 151;
            let mod_line;
            let stat_text = body.split('\n').nth(151 - 1).unwrap();

            if stat_text.is_empty() {
                mod_line = stat_line + 18;
            } else if stat_text == "<h1>Module Search</h1>" {
                return Err(crate::scraper::Error::NotFound);
            } else {
                mod_line = stat_line + 7
            }

            let mod_id: u32 = body
                .split('\n')
                .nth(mod_line - 1)
                .unwrap()
                .split("&amp;query=")
                .nth(1)
                .unwrap()
                .split("\" title")
                .next()
                .unwrap()
                .parse()
                .unwrap();

            Ok(mod_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scraper::resolver::resolve_mod_filename;
    use crate::scraper::ModInfo;

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

    /*
    #[test]
    fn invalid_modid_struct() {
        let invalid = get_full_details_as_struct(30638);
        assert_eq!(invalid.mod_status, "absent");
    }

    #[test]
    fn valid_modid_struct() {
        let valid = get_full_details_as_struct(99356);
        assert_eq!(valid.mod_status, "present");
    }

    #[test]
    fn spotlit_modid_struct() {
        let module = get_full_details_as_struct(158263);
        assert_eq!(module.mod_spotlit, true);
    }
    */

    #[test]
    fn name_resolving() {
        let mod_id = resolve_mod_filename("virtual-monotone.mod");
        assert_eq!(mod_id.unwrap(), 88676);
    }
}
