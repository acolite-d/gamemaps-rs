use gamemaps;

fn main() {
    let gamemap =
        gamemaps::read("MAPHEAD.WL6", "GAMEMAPS.WL6").expect("Should be able to read and init");

    for level in gamemap.levels() {
        println!(
            "Level '{}'\n\tDimensions: {} by {}",
            level.name, level.width, level.height,
        );
    }
}
