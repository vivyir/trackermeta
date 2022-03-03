use trackermeta::ModInfo;

fn main() {
    let modid = ModInfo::resolve_filename("noway.s3m").unwrap()[0].id;
    let modinfo = ModInfo::get(modid).unwrap();
    println!("{:#?}", modinfo);
}
