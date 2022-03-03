use trackermeta::scraper::{resolver, ModInfo};

fn main() {
    let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
    let modtext = ModInfo::get(modid).unwrap();
    println!("{}", modtext.instrument_text);
}
