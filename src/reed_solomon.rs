use crate::config::ByteNameOfMode;
use crate::config::ErrorLevel;
use crate::config::Mask;
use crate::config::BLANK_FILLER;
use crate::config::FORMAT_STRING_XOR_VALUE;
use crate::config::LEVEL_INDICATOR;
use crate::config::MODE_INDICATOR;
use crate::config::REMINDER;
use crate::polynomial::div_polynomial;
use crate::utils;

pub struct ReedSolomonData {
    data: String,
    pub error_level: ErrorLevel,
    pub version: u8,
    pub mask: Mask,
    pub mode: ByteNameOfMode,
    pub bits: String,
}

impl ReedSolomonData {
    pub fn new(
        data: &str,
        min_error_level: ErrorLevel,
        min_version: u8,
        mask: Mask,
        mode: ByteNameOfMode,
    ) -> ReedSolomonData {
        let mut reed_solomon = ReedSolomonData {
            data: String::from(data),
            error_level: ErrorLevel::L,
            version: 1,
            mask: mask,
            mode: mode,
            bits: "".to_string(),
        };
        reed_solomon.get_version_error_level(min_error_level, min_version);
        reed_solomon.generate_data_bits();
        reed_solomon
    }
    pub fn create_format_string(&self) -> String {
        let mask = match self.mask {
            Mask::_000 => "000",
            Mask::_001 => "001",
            Mask::_010 => "010",
            Mask::_011 => "011",
            Mask::_100 => "100",
            Mask::_101 => "101",
            Mask::_110 => "110",
            Mask::_111 => "111",
        };
        let level_mask = format!("{:02b}", LEVEL_INDICATOR[self.error_level as usize]) + &mask;
        let div_format_str = self.main_string_format(&level_mask, 15, 0, 10);
        //println!("•• {:?}", div_format_str);
        let combine_format_str = format!(
            "{:b}",
            u32::from_str_radix(&(level_mask + &div_format_str), 2).unwrap()
                ^ u32::from_str_radix(&FORMAT_STRING_XOR_VALUE, 2).unwrap()
        );
        "0".repeat(15 - combine_format_str.len()) + &combine_format_str
    }
    pub fn create_version_string(&self) -> String {
        let version = format!("{:06b}", self.version);
        let div_format_str = self.main_string_format(&version, 18, 1, 12);
        //println!("• {:?}", div_format_str);
        let combine_format_str = format!(
            "{:b}",
            u32::from_str_radix(&(version + &div_format_str), 2).unwrap()
        );
        "0".repeat(18 - combine_format_str.len()) + &combine_format_str
    }
    pub fn generate_data_bits(&mut self) {
        self.bits = self.create_reed_solomon_matrix().join("")
            + &"0".repeat(REMINDER[(self.version - 1) as usize]);
    }
    fn generate_content(&self) -> Vec<i16> {
        let error_correction_data =
            utils::get_error_correction_level_data(self.version, &self.error_level);
        let msg_len = self.data.len();
        let data_info_bin_len = utils::get_len_bit(&self.mode, self.version);
        let codewords_diff = ((error_correction_data.0 as u16 - (data_info_bin_len as u16 >> 3) - 1)
            as usize)
            - msg_len;
        let prepared_msg_len = String::from(format!("{msg_len:032b}"));
        let msg_data_bin_len = String::from(&prepared_msg_len[32 - data_info_bin_len as usize..]);
        let codewords_data = utils::get_bin_msg_data(&self.data);
        let mut bin_msg = vec![MODE_INDICATOR(&self.mode), msg_data_bin_len];
        for codeword in codewords_data {
            bin_msg.push(codeword);
        }
        bin_msg.push(String::from("0000"));
        for i in 0..codewords_diff {
            bin_msg.push(format!("{:08b}", BLANK_FILLER[(i % 2)]));
        }
        let joined_bin_msg = bin_msg.join("");
        let mut next: usize = 0;
        let mut bin_message_codewords: Vec<i16> = Vec::new();
        for i in 0..(joined_bin_msg.len() / 8) {
            let codeword = &joined_bin_msg[next..(next + 8)];
            next = next + 8;
            let int_value = u8::from_str_radix(codeword, 2).expect("not bin value");
            bin_message_codewords.push(int_value as i16);
        }
        bin_message_codewords
    }
    fn create_reed_solomon_matrix(&self) -> Vec<String> {
        let mut codewords = self.generate_content();
        let error_correction_data =
            utils::get_error_correction_level_data(self.version, &self.error_level);
        let mut groups: Vec<(Vec<i16>, Vec<i16>)> = Vec::new();
        let mut sub: f32 = 0.0;
        let mut error_correction_data_number: u16 = 0;
        for group_number in 0..2 {
            for i in 0..error_correction_data.2[group_number] {
                sub = error_correction_data.3[group_number];
                let mut group = codewords.splice(0..sub as usize, vec![]).collect();
                let mut polynomial = div_polynomial(&mut group, error_correction_data.4 as i16);
                let values: Vec<i16> = polynomial
                    .data
                    .iter_mut()
                    .map(|value| {
                        error_correction_data_number += 1;
                        value.2
                    })
                    .collect();
                groups.push((group, values));
            }
        }

        let msg_codewords_number =
            (2 * error_correction_data.3[0] as u32 * error_correction_data.2[0] as u32
                + error_correction_data.3[1] as u32 * error_correction_data.2[1] as u32)
                as usize;
        let mut msg_codewords: Vec<i16> = vec![257; msg_codewords_number];
        let mut error_correction_codewords: Vec<i16> =
            vec![257; error_correction_data_number as usize];

        for (i, block) in groups.iter().enumerate() {
            let provide_index =
                |index: usize| i + (index + (index * (error_correction_data.1 - 1) as usize));
            for (j, group) in (&block.0).iter().enumerate() {
                msg_codewords[provide_index(j)] = *group;
            }
            for (j, polynomial) in (&block.1).iter().enumerate() {
                error_correction_codewords[provide_index(j)] = *polynomial;
            }
        }
        msg_codewords.append(&mut error_correction_codewords);
        msg_codewords
            .iter_mut()
            .filter_map(|&mut item| {
                if item != 257 {
                    Some(format!("{:08b}", item))
                } else {
                    None
                }
            })
            .collect()
    }
    fn get_version_error_level(&mut self, min_error_level: ErrorLevel, min_version: u8) {
        let len: u32 = self.data.len() as u32;
        let error_levels = vec![ErrorLevel::L, ErrorLevel::Q, ErrorLevel::M, ErrorLevel::H];
        let index_error_level = min_error_level as u8;
        let available_error_levels = &error_levels[index_error_level as usize..];
        for error_level in available_error_levels {
            for version in min_version..=40 {
                if utils::full_capacity(version, &error_level, &self.mode) >= len {
                    self.version = version;
                    self.error_level = *error_level;
                    return;
                }
            }
        }
    }
    fn xor_string_operator(
        &self,
        mut result: String,
        limit: usize,
        generator_polynomial: String,
    ) -> String {
        while result.len() > limit {
            let tmp_generator_polynomial = (*generator_polynomial).to_string()
                + &("0").repeat(result.len() - generator_polynomial.len());
            result = format!(
                "{:b}",
                u32::from_str_radix(&result, 2).unwrap()
                    ^ u32::from_str_radix(&tmp_generator_polynomial, 2).unwrap()
            );
        }
        return result;
    }
    fn main_string_format(
        &self,
        data: &String,
        data_bin_len: u16,
        _type: usize,
        bin_limit: u8,
    ) -> String {
        //println!("*-* {:?}", data_bin_len);
        let prefix_from_string = utils::add_padding_without_prefix(&data, data_bin_len);
        //println!("*** {:?}", prefix_from_string);
        let generator_polynomial =
            utils::get_array_bin_polynomial(_type, bin_limit, String::from(""));
        let result =
            self.xor_string_operator(prefix_from_string, bin_limit.into(), generator_polynomial);
        "0".repeat((bin_limit - result.len() as u8) as usize) + &result
    }
}
