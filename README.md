# Trackermeta

[![license](https://img.shields.io/github/license/vivyir/trackermeta)](https://github.com/vivyir/trackermeta/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/trackermeta)](https://crates.io/crates/trackermeta)
![Crates.io](https://img.shields.io/crates/d/trackermeta)

This is a simple library crate that helps with scraping metadata from the website called [Mod Archive](https://modarchive.org).
It works by parsing the returned HTML and providing the data programmatically, if you have an API key from the Mod Archive please use the XML fork of this library ([Modark](https://github.com/RepellantMold/modark)) by RepellantMold.

Please be sure to donate to [the Mod Archive's hosting fund](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=28NK9DJQRRNGJ) if you use this for any significant amount of time, as scraping data is sure to put strain on their servers and every cent counts!<3

⚠️ This library uses the [`ureq`](https://crates.io/crates/ureq) crate for web requests and isn't back-end agnostic nor asynchronous. Even though `ureq` is a pretty lightweight library if you find a need for those please make an issue!

## Examples

In the following example the program will search for the file name provided by the user and display the data for the closest match.

```rust
use trackermeta::ModInfo;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).unwrap_or(&"".into()).as_ref() {
        "get" => {
            // Returns the first 40 search results, here we'll pick the closest match, if none exist this will panic!
            let mod_id = ModInfo::resolve_filename(
                args.get(2)
                    .expect("No filename provided as second argument."),
            )
            .unwrap()[0].id;

            let mod_info = ModInfo::get(mod_id).unwrap();

            println!("{:#?}", mod_info);
            println!("Download link: {}", mod_info.get_download_link());
        }
        _ => println!("Usage: trackermeta get <filename>"),
    }
}
```

Check out the [examples](examples) directory on the github repo for all examples using the library!

## Roadmap
- Improve code ergonomics and refactor idiomatically
- Better error handling
- Add the ability to traverse paginated searches
- Add more search functions

## License
This project is licenced under the [Mozilla Public License 2.0](https://www.mozilla.org/en-US/MPL/2.0/).
