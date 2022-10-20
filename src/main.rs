mod factorio;

fn main() {
    let mods = factorio::api::get_mods().expect("Failed to get mods from Factorio API");
    mods.results.iter().for_each(|mod_| {
        println!("{}: {}", mod_.name, mod_.title);
    });
}
