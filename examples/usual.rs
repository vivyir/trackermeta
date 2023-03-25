use trackermeta::ModInfo;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).unwrap_or(&"".into()).as_ref() {
        "get" => {
            let mod_id = ModInfo::resolve_filename(
                args.get(2)
                    .expect("No filename provided as second argument."),
            )
            .unwrap()[0]
                .id;
            let mod_info = ModInfo::get(mod_id).unwrap();
            println!("{}", &mod_info.instrument_text);

            println!("\n----------------------------------------\n");

            println!("{:#?}", mod_info);

            println!("\n----------------------------------------\n");

            println!("Download link: {}", mod_info.get_download_link());
        }
        _ => println!("Usage: trackermeta get <filename>"),
    }
}
