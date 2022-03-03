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
            println!("{}", ModInfo::get(mod_id).unwrap().instrument_text);

            println!("\n----------------------------------------\n");

            let mod_info = ModInfo::get(mod_id).unwrap();
            println!("{:#?}", mod_info);
        }
        _ => println!("Usage: trackermeta get <filename>"),
    }
}
