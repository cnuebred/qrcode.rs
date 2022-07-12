use crate::config::ByteNameOfMode;
use crate::config::ErrorLevel;
use crate::config::Mask;
use crate::config::ALIGNMENT_LOCATION_BASE;
use crate::config::BASE_SIZE;
use crate::config::LENGTH_BITS;
use crate::config::VERSION_FORMAT_POLYNOMIAL;
#[path = "config.rs"]
mod config;

pub fn get_len_bit(mode: &ByteNameOfMode, version: u8) -> u8 {
    let number_of_mode: u8 = match mode {
        ByteNameOfMode::Numeric => 1,
        ByteNameOfMode::Alpha => 2,
        ByteNameOfMode::Byte => 4,
        ByteNameOfMode::Kanji => 8,
    };
    let mode_index = 31 - (32 - format!("{number_of_mode:b}").len());
    let bits_index = if version > 26 {
        2
    } else if version > 9 {
        1
    } else {
        0
    };
    LENGTH_BITS[mode_index][bits_index]
}

fn get_total_module_data_version(version: u8) -> u16 {
    if version == 1 {
        return 21 * 21 - 3 * 8 * 8 - 2 * 15 - 1 - 2 * 5;
    }
    let alignment_count = ((version as f32 / 7.0).floor() + 2.0) as u16;
    return ((version * 4 + 17) as i16).pow(2) as u16
        - 3 * 8 * 8 as u16
        - (alignment_count.pow(2) - 3) * 25
        - 2 * (version * 4 + 1) as u16
        + (alignment_count - 2) * 10
        - 2 * 15
        - 1
        - if version > 6 { 2 * 3 * 6 } else { 0 };
}

pub fn get_bin_msg_data(data: &String) -> Vec<String> {
    let data_into_bytes = data.clone().into_bytes();
    let mut bin_msg_data: Vec<String> = Vec::new();
    for char in &data_into_bytes {
        bin_msg_data.push(format!("{char:08b}"));
    }
    bin_msg_data
}

pub fn mask(mask: Mask, col: u16, row: u16) -> bool {
    match mask {
        Mask::_000 => (col + row) % 2 == 0,
        Mask::_001 => row % 2 == 0,
        Mask::_010 => col % 3 == 0,
        Mask::_011 => (col + row) % 3 == 0,
        Mask::_100 => ((col as f32 / 3.).floor() + (row as f32 / 2.).floor()) % 2.0 == 0.,
        Mask::_101 => (((row * col) % 2) + ((row * col) % 3)) == 0,
        Mask::_110 => (((row * col) % 2) + ((row * col) % 3)) % 2 == 0,
        Mask::_111 => (((row + col) % 2) + ((row * col) % 3)) % 2 == 0,
    }
}

pub fn get_alignment(mut version: u8) -> Vec<u32> {
    version = version - 2;
    let version_align: u32 = version as u32 * 4;
    let mut align: Vec<u32> = ALIGNMENT_LOCATION_BASE.to_vec();
    let len = align.len();
    align[&len - 1] = ALIGNMENT_LOCATION_BASE[1] + version_align;

    let align_count =
        ((((BASE_SIZE + version_align - 4) - ALIGNMENT_LOCATION_BASE[0]) as f32 / 14.0) / 2.0)
            .ceil() as u32;

    let mut diff = ((ALIGNMENT_LOCATION_BASE[1] + version_align - align[0]) as f32
        / align_count as f32)
        .ceil() as u32;
    if diff % 2 != 0 {
        diff += 1
    }
    for i in 2..7 {
        if align_count >= i {
            let removed = align.len() as u32 - (i - 1);
            align.insert(
                removed as usize,
                align[removed as usize] as u32 - diff as u32,
            )
        }
    }
    return align;
}

pub fn get_codewords_number(version: u8, error_level: &ErrorLevel) -> u32 {
    let index_error_level: usize = match error_level {
        ErrorLevel::L => 0,
        ErrorLevel::M => 1,
        ErrorLevel::Q => 2,
        ErrorLevel::H => 3,
    };
    let (error_codewords_per_block, block_number): (u8, u8) =
        config::TABLE_EC[(version - 1) as usize][index_error_level as usize];
    (get_total_module_data_version(version) >> 3) as u32
        - (error_codewords_per_block as u32 * block_number as u32) as u32
}

pub fn get_array_bin_polynomial(version_size: usize, i: u8, mut _str: String) -> String {
    _str += if VERSION_FORMAT_POLYNOMIAL[version_size].contains(&i) {
        &"1"
    } else {
        &"0"
    };
    if i <= 0 {
        return _str;
    }
    get_array_bin_polynomial(version_size, i - 1, _str)
}

pub fn add_padding_without_prefix(string: &String, padding_size: u16) -> String {
    let padding = ("0".repeat(padding_size as usize)).to_string();
    return (format!(
        "{:b}",
        u32::from_str_radix(&(string.to_owned() + &padding)[0..padding_size as usize], 2).unwrap()
    ))
    .to_string();
}

pub fn capacity(bits: u32) -> Vec<u32> {
    vec![
        bits >> 3 as u32,
        ((bits / 10 * 3)
            + if bits % 10 > 6 {
                2
            } else if bits % 10 > 3 {
                1
            } else {
                0
            }) as u32,
        ((bits / 11 as u32) * 2 + if bits % 11 > 5 { 1 } else { 0 }) as u32,
        (bits / 13) as u32,
    ]
}

pub fn full_capacity(version: u8, error_level: &ErrorLevel, mode: &ByteNameOfMode) -> u32 {
    let codewords_number = get_codewords_number(version, error_level);
    let free_modules = (codewords_number << 3) - get_len_bit(&mode, version) as u32;
    let number_of_mode: u8 = match mode {
        ByteNameOfMode::Byte => 0,
        ByteNameOfMode::Numeric => 1,
        ByteNameOfMode::Alpha => 2,
        ByteNameOfMode::Kanji => 3,
    };
    capacity(free_modules)[number_of_mode as usize]
}

pub fn get_error_correction_level_data(
    version: u8,
    error_level: &ErrorLevel,
) -> (u16, u16, [u16; 2], [f32; 2], u16) {
    if version > 40 {
        panic!("Version is to big")
    }
    let index_error_level: usize = match error_level {
        ErrorLevel::L => 0,
        ErrorLevel::M => 1,
        ErrorLevel::Q => 2,
        ErrorLevel::H => 3,
    };
    let (error_codewords_per_block, block_number): (u8, u8) =
        config::TABLE_EC[(version - 1) as usize][index_error_level as usize];
    let total_modules: u16 = (get_total_module_data_version(version) >> 3) as u16;
    let second_group = total_modules % block_number as u16;
    let codewords: u16 = total_modules - error_codewords_per_block as u16 * block_number as u16;
    let groups: [u16; 2] = [block_number as u16 - second_group, second_group];
    let codewords_in_group = (codewords as f32 / block_number as f32).floor();
    (
        codewords,
        block_number as u16,
        groups,
        [codewords_in_group, codewords_in_group + 1.0],
        error_codewords_per_block as u16,
    )
}
