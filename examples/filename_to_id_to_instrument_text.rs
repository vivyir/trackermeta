use trackermeta::scraper::{requests, resolver};

fn main() {
    let modid = resolver::resolve_mod_filename("noway.s3m").unwrap();
    let modtext = requests::get_instrument_text(modid).unwrap();
    println!("{}", modtext);
}
