use trackermeta::ModInfo;

fn main() {
    let modid = ModInfo::resolve_filename("noway.s3m").unwrap()[0].id;
    let modtext = ModInfo::get(modid).unwrap();
    println!("{}", modtext.instrument_text);
}
