use chrono::prelude::{DateTime, Utc};

// https://stackoverflow.com/a/64148190
fn iso8601_time(st: &std::time::SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    format!("{}", dt.format("%+"))
}

pub mod trackermeta {
    use crate::iso8601_time;

    #[derive(Debug)]
    pub enum Error {
        NotFound,
    }

    pub fn get_full_details(module_id: u32) -> String {
        // priv info
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

        let body: String = ureq::get(
            format!(
                "https://modarchive.org/index.php?request=view_by_moduleid&query={}",
                mod_id
            )
            .as_str(),
        )
        .call()
        .unwrap()
        .into_string()
        .unwrap();

        let mod_status_text = body.split('\n').nth(184 - 1).unwrap();
        if mod_status_text == "" {
            mod_status = "absent";
        }

        if mod_status != "absent" {
            {
                let mod_spotlit_text = body.split('\n').nth(170 - 1).unwrap();
                if mod_spotlit_text != "" {
                    mod_spotlit = true;
                    mod_filename_line = mod_filename_line + 6;
                    mod_download_line = mod_download_line + 6;
                    mod_info_line = mod_info_line + 6;
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
                .split("#")
                .nth(1)
                .unwrap()
                .split("\">")
                .nth(0)
                .unwrap();

            mod_download = body
                .split('\n')
                .nth(mod_download_line - 1)
                .unwrap()
                .split("Downloads: ")
                .nth(1)
                .unwrap()
                .split("</li>")
                .nth(0)
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
                .nth(0)
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
                .nth(0)
                .unwrap();

            mod_channel = body
                .split('\n')
                .nth(mod_channel_line - 1)
                .unwrap()
                .split("Channels: ")
                .nth(1)
                .unwrap()
                .split("</li>")
                .nth(0)
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
                .nth(0)
                .unwrap();

            mod_genre = body
                .split('\n')
                .nth(mod_genre_line - 1)
                .unwrap()
                .split("Genre: ")
                .nth(1)
                .unwrap()
                .split("</li>")
                .nth(0)
                .unwrap();

            mod_upload_date = body
                .split('\n')
                .nth(mod_info_line - 1)
                .unwrap()
                .split("</b> times since ")
                .nth(1)
                .unwrap()
                .split(" :D")
                .nth(0)
                .unwrap();

            mod_format = mod_filename
                .clone()
                .split('.')
                .nth(1)
                .unwrap()
                .to_uppercase();
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
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            mod_id,
            mod_status,
            mod_filename,
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

    pub fn resolve_mod_filename(mod_filename: &str) -> Result<u32, crate::trackermeta::Error> {
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

        if stat_text == "" {
            mod_line = stat_line + 18;
        } else if stat_text == "<h1>Module Search</h1>" {
            return Err(crate::trackermeta::Error::NotFound);
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
            .nth(0)
            .unwrap()
            .parse()
            .unwrap();

        Ok(mod_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::trackermeta::*;

    #[test]
    fn invalid_modid() {
        let invalid = get_full_details(30638);
        assert_eq!(invalid.split(',').nth(1).unwrap(), "absent");
    }

    #[test]
    fn valid_modid() {
        let valid = get_full_details(99356);
        assert_eq!(valid.split(',').nth(1).unwrap(), "present");
    }

    #[test]
    fn spotlit_modid() {
        let module = get_full_details(158263);
        assert_eq!(module.split(',').nth(6).unwrap(), "true");
    }

    #[test]
    fn name_resolving() {
        let mod_id = resolve_mod_filename("virtual-monotone.mod");
        assert_eq!(mod_id.unwrap(), 88676);
    }
}
