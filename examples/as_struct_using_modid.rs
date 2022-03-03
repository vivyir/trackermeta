use trackermeta::ModInfo;

fn main() {
    let modinfo = ModInfo::get(51772).unwrap();
    println!("{:#?}", modinfo);
}
