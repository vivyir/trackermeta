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
//! use trackermeta::scraper::requests;
//!
//! fn main() {
//!     let modinfo = requests::get_full_details_as_struct(51772);
//!     println!("{:#?}", modinfo);
//! }
//! ```
//!
//! ## Example: Resolve filename to id then use id to get the info as struct
//! ```rust
//! use trackermeta::scraper::{requests, resolver};
//!
//! fn main() {
//!     let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
//!     let modinfo = requests::get_full_details_as_struct(modid);
//!     println!("{:#?}", modinfo);
//! }
//! ```
//!
//! ## Example: Resolve filename to id then use id to get the info as string
//! ```rust
//! use trackermeta::scraper::{requests, resolver};
//!
//! fn main() {
//!     let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
//!     let modinfo = requests::get_full_details_as_string(modid);
//!     println!("{}", modinfo);
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

    /// Error enum for functions in the crate that return a [`Result`]
    #[derive(Debug)]
    pub enum Error {
        NotFound,
    }

    /// Struct containing all of the info about a module
    #[derive(Debug)]
    pub struct ModInfo {
        /// The module ID of the module on modarchive
        pub info_mod_id: u32,
        /// Can be either `absent` or `present`
        pub info_mod_status: String,
        /// The filename of the module
        pub info_mod_filename: String,
        /// The title of the module
        pub info_mod_title: String,
        /// The file size of the module, use the
        /// crate `byte-unit` to convert them to other units
        pub info_mod_size: String,
        /// The MD5 hash of the module file as a string
        pub info_mod_md5: String,
        /// The format of the module, for example `XM`, `IT`
        /// or `MOD` and more, basically the extension of the
        /// module file
        pub info_mod_format: String,
        /// Spotlit module or not
        pub info_mod_spotlit: bool,
        /// Download count of the module at the time of scraping
        pub info_mod_download: u32,
        /// Times the module has been favourited at the time of
        /// scraping
        pub info_mod_fav: u32,
        /// The time when it was scraped
        pub info_mod_scrape_time: String,
        /// The channel count of the module
        pub info_mod_channel: u32,
        /// The genre of the module
        pub info_mod_genre: String,
        /// The upload date of the module
        pub info_mod_upload_date: String,
    }

    /// Module containing scraper requests you can make to modarchive
    pub mod requests {
        use crate::iso8601_time;
        use crate::scraper::ModInfo;

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

        /// Get every detail about a module and return a [`String`]
        ///
        /// The string returned is formatted using the [`format!()`] macro
        /// like so:
        /// ```rust
        /// format!(
        ///    "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        ///     mod_id,
        ///     mod_status,
        ///     mod_filename,
        ///     mod_title,
        ///     mod_size,
        ///     mod_md5,
        ///     mod_format,
        ///     mod_spotlit,
        ///     mod_download,
        ///     mod_fav,
        ///     mod_scrape_time,
        ///     mod_channel,
        ///     mod_genre,
        ///     mod_upload_date
        /// )
        /// ```
        /// And an example string returned would be:
        ///
        /// `51772,present,noway.s3m,No Way To Know,140.78KB,ebbf37d7c868f3c3c551702711fe8512,S3M,false,1938,0,2021-07-13T09:07:50.641382301+00:00,10,Trance - Dream,Fri 27th Nov 1998`
        pub fn get_full_details_as_string(module_id: u32) -> String {
            // priv info
            let mod_title_line = 140; // as long as the modarchive toolbar doesnt change this is stable
            let mut mod_filename_line = 178;
            let mut mod_info_line = 192;
            let mut mod_download_line = 198;
            let mut mod_fav_line = mod_download_line + 1;
            let mut mod_md5_line = mod_fav_line + 1;
            let mut mod_channel_line = mod_md5_line + 2;
            let mut mod_size_line = mod_channel_line + 1;
            let mut mod_genre_line = mod_size_line + 1;

            // front facing info
            let mut mod_spotlit = false;
            let mut mod_format = String::from("invalid");
            let mut mod_filename = "invalid";
            let mut mod_title = String::from("invalid");
            let mut mod_status = "present";
            let mut mod_upload_date = "Thu 1st Jan 1970";
            let mut mod_download: u32 = 0;
            let mut mod_fav: u32 = 0;
            let mut mod_md5 = "00000000000000000000000000000000";
            let mut mod_channel: u32 = 0;
            let mut mod_size = "0B";
            let mut mod_genre = "n/a";
            let mod_scrape_time = iso8601_time(&std::time::SystemTime::now());

            // special info
            let mod_id = module_id;

            #[cfg(feature = "overridable")]
            {
                let line_tuple = load_lines(mod_filename_line, mod_info_line, mod_download_line);
                mod_filename_line = line_tuple.0;
                mod_info_line = line_tuple.1;
                mod_download_line = line_tuple.2;
            }

            let body: String = inner_request(mod_id);

            let mod_status_text = body.split('\n').nth(184 - 1).unwrap();
            if mod_status_text.is_empty() {
                mod_status = "absent";
            }

            if mod_status != "absent" {
                {
                    let mod_spotlit_text = body.split('\n').nth(170 - 1).unwrap();
                    if !mod_spotlit_text.is_empty() {
                        mod_spotlit = true;
                        mod_filename_line += 6;
                        mod_download_line += 6;
                        mod_info_line += 6;
                        mod_fav_line = mod_download_line + 1;
                        mod_md5_line = mod_fav_line + 1;
                        mod_channel_line = mod_md5_line + 2;
                        mod_size_line = mod_channel_line + 1;
                        mod_genre_line = mod_size_line + 1;
                    }
                }

                mod_filename = body
                    .split('\n')
                    .nth(mod_filename_line - 1)
                    .unwrap()
                    .split('#')
                    .nth(1)
                    .unwrap()
                    .split("\">")
                    .next()
                    .unwrap();

                mod_title = escaper::decode_html(
                    body.split('\n')
                        .nth(mod_title_line - 1)
                        .unwrap()
                        .split("<h1>")
                        .nth(1)
                        .unwrap()
                        .split(" <span class")
                        .next()
                        .unwrap(),
                )
                .unwrap();

                mod_download = body
                    .split('\n')
                    .nth(mod_download_line - 1)
                    .unwrap()
                    .split("Downloads: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();

                mod_fav = body
                    .split('\n')
                    .nth(mod_fav_line - 1)
                    .unwrap()
                    .split("Favourited: ")
                    .nth(1)
                    .unwrap()
                    .split(" times</li>")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();

                mod_md5 = body
                    .split('\n')
                    .nth(mod_md5_line - 1)
                    .unwrap()
                    .split("MD5: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap();

                mod_channel = body
                    .split('\n')
                    .nth(mod_channel_line - 1)
                    .unwrap()
                    .split("Channels: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();

                mod_size = body
                    .split('\n')
                    .nth(mod_size_line - 1)
                    .unwrap()
                    .split("Uncompressed Size: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap();

                mod_genre = body
                    .split('\n')
                    .nth(mod_genre_line - 1)
                    .unwrap()
                    .split("Genre: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap();

                mod_upload_date = body
                    .split('\n')
                    .nth(mod_info_line - 1)
                    .unwrap()
                    .split("</b> times since ")
                    .nth(1)
                    .unwrap()
                    .split(" :D")
                    .next()
                    .unwrap();

                mod_format = mod_filename.split('.').nth(1).unwrap().to_uppercase();
            }

            // formatted nicely
            /*
                println!(
                    r#"
            Module ID: {}
            Mod status: {}
            Spotlit: {}
            Scraped at: {}
            Filename: {}
            Format: {}
            Downloaded: {} times
            Favourited: {} times
            MD5: {}
            Channels: {}
            Size: {}
            Genre: {}
            Upload date: {}
                    "#,
                    mod_id,
                    mod_status,
                    mod_spotlit,
                    mod_scrape_time,
                    mod_filename,
                    mod_format,
                    mod_download,
                    mod_fav,
                    mod_md5,
                    mod_channel,
                    mod_size,
                    mod_genre,
                    mod_upload_date
                );
                */

            // csv style
            format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                mod_id,
                mod_status,
                mod_filename,
                mod_title,
                mod_size,
                mod_md5,
                mod_format,
                mod_spotlit,
                mod_download,
                mod_fav,
                mod_scrape_time,
                mod_channel,
                mod_genre,
                mod_upload_date
            )
        }

        /// Get every detail about a module and return a [`ModInfo`]
        pub fn get_full_details_as_struct(module_id: u32) -> ModInfo {
            // priv info
            let mod_title_line = 140; // as long as the modarchive toolbar doesnt change this is stable
            let mut mod_filename_line = 178;
            let mut mod_info_line = 192;
            let mut mod_download_line = 198;
            let mut mod_fav_line = mod_download_line + 1;
            let mut mod_md5_line = mod_fav_line + 1;
            let mut mod_channel_line = mod_md5_line + 2;
            let mut mod_size_line = mod_channel_line + 1;
            let mut mod_genre_line = mod_size_line + 1;

            // front facing info
            let mut mod_spotlit = false;
            let mut mod_format = "invalid".to_string();
            let mut mod_filename = "invalid";
            let mut mod_title = String::from("invalid");
            let mut mod_status = "present";
            let mut mod_upload_date = "Thu 1st Jan 1970";
            let mut mod_download: u32 = 0;
            let mut mod_fav: u32 = 0;
            let mut mod_md5 = "00000000000000000000000000000000";
            let mut mod_channel: u32 = 0;
            let mut mod_size = "0B";
            let mut mod_genre = "n/a";
            let mod_scrape_time = iso8601_time(&std::time::SystemTime::now());

            // special info
            let mod_id = module_id;

            #[cfg(feature = "overridable")]
            {
                let line_tuple = load_lines(mod_filename_line, mod_info_line, mod_download_line);
                mod_filename_line = line_tuple.0;
                mod_info_line = line_tuple.1;
                mod_download_line = line_tuple.2;
            }

            let body: String = inner_request(mod_id);

            let mod_status_text = body.split('\n').nth(184 - 1).unwrap();
            if mod_status_text.is_empty() {
                mod_status = "absent";
            }

            if mod_status != "absent" {
                {
                    let mod_spotlit_text = body.split('\n').nth(170 - 1).unwrap();
                    if !mod_spotlit_text.is_empty() {
                        mod_spotlit = true;
                        mod_filename_line += 6;
                        mod_download_line += 6;
                        mod_info_line += 6;
                        mod_fav_line = mod_download_line + 1;
                        mod_md5_line = mod_fav_line + 1;
                        mod_channel_line = mod_md5_line + 2;
                        mod_size_line = mod_channel_line + 1;
                        mod_genre_line = mod_size_line + 1;
                    }
                }

                mod_filename = body
                    .split('\n')
                    .nth(mod_filename_line - 1)
                    .unwrap()
                    .split('#')
                    .nth(1)
                    .unwrap()
                    .split("\">")
                    .next()
                    .unwrap();

                mod_title = escaper::decode_html(
                    body.split('\n')
                        .nth(mod_title_line - 1)
                        .unwrap()
                        .split("<h1>")
                        .nth(1)
                        .unwrap()
                        .split(" <span class")
                        .next()
                        .unwrap(),
                )
                .unwrap();

                mod_download = body
                    .split('\n')
                    .nth(mod_download_line - 1)
                    .unwrap()
                    .split("Downloads: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();

                mod_fav = body
                    .split('\n')
                    .nth(mod_fav_line - 1)
                    .unwrap()
                    .split("Favourited: ")
                    .nth(1)
                    .unwrap()
                    .split(" times</li>")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();

                mod_md5 = body
                    .split('\n')
                    .nth(mod_md5_line - 1)
                    .unwrap()
                    .split("MD5: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap();

                mod_channel = body
                    .split('\n')
                    .nth(mod_channel_line - 1)
                    .unwrap()
                    .split("Channels: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();

                mod_size = body
                    .split('\n')
                    .nth(mod_size_line - 1)
                    .unwrap()
                    .split("Uncompressed Size: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap();

                mod_genre = body
                    .split('\n')
                    .nth(mod_genre_line - 1)
                    .unwrap()
                    .split("Genre: ")
                    .nth(1)
                    .unwrap()
                    .split("</li>")
                    .next()
                    .unwrap();

                mod_upload_date = body
                    .split('\n')
                    .nth(mod_info_line - 1)
                    .unwrap()
                    .split("</b> times since ")
                    .nth(1)
                    .unwrap()
                    .split(" :D")
                    .next()
                    .unwrap();

                mod_format = mod_filename.split('.').nth(1).unwrap().to_uppercase();
            }

            ModInfo {
                info_mod_id: mod_id,
                info_mod_status: mod_status.into(),
                info_mod_filename: mod_filename.into(),
                info_mod_title: mod_title,
                info_mod_size: mod_size.into(),
                info_mod_md5: mod_md5.into(),
                info_mod_format: mod_format,
                info_mod_spotlit: mod_spotlit,
                info_mod_download: mod_download,
                info_mod_fav: mod_fav,
                info_mod_scrape_time: mod_scrape_time,
                info_mod_channel: mod_channel,
                info_mod_genre: mod_genre.into(),
                info_mod_upload_date: mod_upload_date.into(),
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
    use crate::scraper::requests::{
        get_full_details_as_string, get_full_details_as_struct, get_instrument_text,
    };
    use crate::scraper::resolver::resolve_mod_filename;

    #[test]
    fn instr_text() {
        let instr_text = get_instrument_text(61772).unwrap();
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
        let invalid = get_full_details_as_string(30638);
        assert_eq!(invalid.split(',').nth(1).unwrap(), "absent");
    }

    #[test]
    fn valid_modid() {
        let valid = get_full_details_as_string(99356);
        assert_eq!(valid.split(',').nth(1).unwrap(), "present");
    }

    #[test]
    fn spotlit_modid() {
        let module = get_full_details_as_string(158263);
        assert_eq!(module.split(',').nth(7).unwrap(), "true");
    }

    #[test]
    fn invalid_modid_struct() {
        let invalid = get_full_details_as_struct(30638);
        assert_eq!(invalid.info_mod_status, "absent");
    }

    #[test]
    fn valid_modid_struct() {
        let valid = get_full_details_as_struct(99356);
        assert_eq!(valid.info_mod_status, "present");
    }

    #[test]
    fn spotlit_modid_struct() {
        let module = get_full_details_as_struct(158263);
        assert_eq!(module.info_mod_spotlit, true);
    }

    #[test]
    fn name_resolving() {
        let mod_id = resolve_mod_filename("virtual-monotone.mod");
        assert_eq!(mod_id.unwrap(), 88676);
    }
}
