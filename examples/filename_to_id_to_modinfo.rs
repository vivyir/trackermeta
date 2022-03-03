use trackermeta::scraper::{resolver, ModInfo};

fn main() {
    let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
    let modinfo = ModInfo::get(modid).unwrap();
    println!("{:#?}", modinfo);
}
