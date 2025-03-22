use itertools::Itertools;
use std::{error::Error, fs, io::Read, path::Path, ptr};

// GAMEMAPS ON-WIRE STRUCTS

#[repr(packed)]
#[derive(Debug, PartialEq, Eq)]
struct PackedMapHeader {
    magic: u16,
    levels_offsets: [i32; 100],
}

#[repr(packed)]
#[derive(Debug, PartialEq, Eq)]
struct PackedLevelHeader {
    plane_offsets: [i32; 3],
    plane_lengths: [u16; 3],
    width: u16,
    height: u16,
    name: [u8; 16],
}

// FORWARD FACING API STRUCTS AND FUNCTIONS

#[derive(Debug, PartialEq, Eq)]
pub struct GameData {
    maps_data: Box<[u8]>,
    level_offsets: [i32; 100],
    magic: u16,
    tileinfo: Option<Box<[u8]>>,
}

pub fn read<P: AsRef<Path>>(header_file: P, maps_file: P) -> Result<GameData, Box<dyn Error>> {
    let (hdr, tileinfo) = fs::File::open(header_file).map(|mut fp| {
        let mut buf: Vec<u8> = vec![];
        let sz = fp.read_to_end(&mut buf).unwrap();

        match sz {
            38 => {
                let hdr: PackedMapHeader = unsafe { buf.as_ptr().cast::<PackedMapHeader>().read() };
                (hdr, None)
            }

            39.. => {
                let hdr: PackedMapHeader = unsafe { buf.as_ptr().cast::<PackedMapHeader>().read() };

                let tileinfo = buf[38..].iter().copied().collect::<Box<[u8]>>();

                (hdr, Some(tileinfo))
            }

            0..=37 => {
                panic!("Cannot read header file, too small, less than minimum 38 bytes in size");
            }
        }
    })?;

    let maps_buf = fs::read(maps_file).map(|vec| vec.into_boxed_slice())?;

    let game_data = unsafe {
        GameData {
            maps_data: maps_buf,
            level_offsets: *ptr::addr_of!(hdr.levels_offsets),
            magic: *ptr::addr_of!(hdr.magic),
            tileinfo,
        }
    };

    // TODO:
    // Possibly check maximum level offset and see if it is less than size of map maps_data
    // if not, then the level offsets found in header are incorrect for this map file,
    // propogate error

    Ok(game_data)
}

impl GameData {
    pub fn levels(&self) -> Levels {
        // Iterate over level offsets, read in level headers, provide references to level data
        Levels {
            maps_data: self.maps_data.as_ref(),
            offsets: self.level_offsets.iter(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Level<'gm> {
    pub planes: [&'gm [u8]; 3],
    pub name: &'gm str,
    pub width: u16,
    pub height: u16,
}

pub struct Planes {}

#[derive(Debug)]
pub struct Levels<'gm> {
    maps_data: &'gm [u8],
    offsets: std::slice::Iter<'gm, i32>,
}

impl<'gm> Iterator for Levels<'gm> {
    type Item = Level<'gm>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = *self.offsets.next().filter(|&off| *off > 0)?;

        let hdr = unsafe {
            self.maps_data
                .as_ptr()
                .add(offset as usize)
                .cast::<PackedLevelHeader>()
                .read()
        };

        let level = unsafe {
            // GameMaps level names are ASCII strings, ASCII subset of UTF-8, should
            // be good to just convert this unchecked
            let name = str::from_utf8_unchecked(&*ptr::addr_of!(hdr.name));

            let offsets = ptr::addr_of!(hdr.plane_offsets)
                .read()
                .into_iter()
                .map(|n| n as usize)
                .collect_tuple::<(usize, usize, usize)>()
                .unwrap();

            let lens = ptr::addr_of!(hdr.plane_lengths)
                .read()
                .into_iter()
                .map(|n| n as usize)
                .collect_tuple::<(usize, usize, usize)>()
                .unwrap();

            Level {
                planes: [
                    &self.maps_data[offsets.0..offsets.0 + lens.0],
                    &self.maps_data[offsets.1..offsets.1 + lens.1],
                    &self.maps_data[offsets.2..offsets.2 + lens.2],
                ],
                width: *ptr::addr_of!(hdr.width),
                height: *ptr::addr_of!(hdr.height),
                name,
            }
        };

        Some(level)
    }
}

// Possible Builder API, maybe include it and expand it later
// #[derive(Debug, Default)]
// struct Builder {
//     maps_data: Box<[u8]>,
//     level_offsets: [i32; 100],
//     magic: u16,
//     tileinfo: Option<Box<[u8]>>,
// }
//
// impl Builder {
//     pub fn new() -> Self {
//         Builder::default()
//     }
//
//     pub fn header_file(mut self, file: &Path) -> Self {}
//
//     pub fn maps_file(mut self, file: &Path) -> Self {
//         todo!();
//     }
//
//     pub fn build() -> Result<Self, Box<dyn Error>> {}
// }
