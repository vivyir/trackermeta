use trackermeta::scraper::{requests, resolver};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).unwrap_or(&"".into()).as_ref() {
        "get" => {
            let mod_id = resolver::resolve_mod_filename(
                args.get(2)
                    .expect("No filename provided as second argument."),
            )
            .unwrap();
            let mod_info = requests::get_full_details_as_struct(mod_id);
            println!("{:#?}", mod_info);
            /* or if you want it pre-formatted as csv:
            let mod_info = requests::get_full_details_as_string(mod_id);
            println!("{}", mod_info);
            */
        }
        _ => println!("Usage: trackermeta get <filename>"),
    }
}
