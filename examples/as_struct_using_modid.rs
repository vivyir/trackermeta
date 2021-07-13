use trackermeta::scraper::requests;

fn main() {
    let modinfo = requests::get_full_details_as_struct(51772);
    println!("{:#?}", modinfo);
}
