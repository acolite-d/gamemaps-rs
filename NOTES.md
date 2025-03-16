# IDEA FOR THE GAMEMAPS API


```rust
use gamemaps;

let game_data = gamemaps::read()

// Find a particular level
let e2m3: Level<'gm> = game_data.levels().find(|level| level.name == "xxx")
	.expect("This map should have it");

// Uncompressed plane
// This wont work unless we pass down references to the entire map data buffer
// to Level<'_>, which seems awkward?
let background_plane: &[u8] = e2m3.planes().1; 

// Alternatively...
let background_plane: PlaneRef = e2m3.planes().1;
let plane_data: &'gm [u8] = game_data.retrieve_plane_compressed(background_plane);

// For deflated data
let plane_data: Box<[u8]> = game_data.retrieve_plane_deflated(background_plane);

// Another way would be to put both iterators in same GameData impl
let levels = game_data.levels.collect();
let planes = game_data.planes.collect();

// Have an ID associated with planes so you can trace back to a level
// Or to prevent duplication of data across levels and planes...

let (l, planes): (Level<'gm>, ()) = game_data.levels_and_planes().find(|| ...) 

```



