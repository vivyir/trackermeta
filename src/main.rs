use trackermeta::trackermeta;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).unwrap_or(&"".into()).as_ref() {
        "get" => {
            let mod_id = trackermeta::resolve_mod_filename(args.get(2).expect("No filename provided as second argument.")).unwrap();
            let mod_info = trackermeta::get_full_details(mod_id);
            println!("{}", mod_info);
        }
        _ => println!("Usage: trackermeta get <filename>"),
    }
}
