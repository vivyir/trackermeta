use trackermeta::scraper::{requests, resolver};

fn main() {
    let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
    let modinfo = requests::get_full_details_as_string(modid);
    println!("{}", modinfo);
}
