#[path = "reed_solomon.rs"]
mod reed_solomon;
use crate::config::ByteNameOfMode;
use crate::config::ErrorLevel;
use crate::config::Mask;
use crate::utils::get_alignment;
use crate::utils::mask;
use reed_solomon::ReedSolomonData;
use std::fmt;
pub struct Matrix<T> {
    size_x: u32,
    size_y: u32,
    matrix: Vec<Vec<T>>,
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.size_y {
            let mut row: String = "|".to_string();
            for j in 0..self.size_x {
                row += &String::from(format!("{:?}|", self.matrix[i as usize][j as usize]))
            }
            println!("{:?}", row);
        }
        write!(f, "")
    }
}
impl fmt::Debug for QRcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.qrcode.size_y {
            let mut row: String = "".to_string();
            for j in 0..self.qrcode.size_x {
                row += if format!("{:?}", self.qrcode.matrix[i as usize][j as usize]) == "1" {
                    "██"
                } else {
                    if format!("{:?}", self.qrcode.matrix[i as usize][j as usize]) == "0" {
                        "  "
                    } else {
                        "░░"
                    }
                }
            }
            println!("{:?}", row);
        }
        write!(f, "")
    }
}

impl<T> Matrix<T>
where
    T: Copy + Into<T> + std::fmt::Debug,
{
    pub fn create(size_x: u32, size_y: u32, filler: &T) -> Matrix<T> {
        let mut vec_table: Vec<Vec<T>> = Vec::with_capacity((size_y) as usize);
        for i in 0..(size_y) {
            let mut vec_row: Vec<T> = Vec::with_capacity((size_x) as usize);
            for i in 0..(size_x) {
                vec_row.push(*filler);
            }
            vec_table.push(vec_row);
        }
        Matrix::<T> {
            size_x: size_x,
            size_y: size_y,
            matrix: vec_table,
        }
    }
    pub fn put(&mut self, point: (u32, u32), value: T) {
        self.matrix[point.1 as usize][point.0 as usize] = value
    }
    pub fn get(&self, point: (u32, u32)) -> T {
        return self.matrix[point.1 as usize][point.0 as usize];
    }
    pub fn transpose(&mut self) {
        let mut new_matrix: Matrix<T> =
            Matrix::create(self.size_y, self.size_x, &self.matrix[0][0]);
        for i in 0..new_matrix.size_y {
            for j in 0..new_matrix.size_x {
                new_matrix.put((j, i), self.get((i, j)))
            }
        }
        *self = new_matrix;
    }
    pub fn put_vec(&mut self, point: (u32, u32), vec: &Vec<T>, horizontal: bool) {
        for i in 0..vec.len() {
            if horizontal {
                self.matrix[point.1 as usize][point.0 as usize + i] = vec[i];
            } else {
                self.matrix[point.1 as usize + i][point.0 as usize] = vec[i];
            }
        }
    }
    pub fn put_matrix(&mut self, point: (u32, u32), matrix: &Matrix<T>) {
        if point.0 + matrix.size_x > self.size_x || point.1 + matrix.size_y > self.size_y {
            panic!("matrix to put is too big");
        }
        for i in 0..matrix.size_y {
            for j in 0..matrix.size_x {
                self.put((j + point.0, i + point.1), matrix.get((j, i)))
            }
        }
    }
    pub fn rotate(&mut self) {
        let len = self.size_x - 1;
        if self.size_y == (len + 1) {
            let i_range = (len - 1 / 2) as f32;
            for i in 0..i_range.floor() as u32 {
                let part_1: f32 = (len - 1) as f32 / 2 as f32;
                let part_2: f32 = (len + 1) as f32 / 2 as f32;

                for j in i..(part_1.floor() as u32 + part_2.ceil() as u32 - i) {
                    let base: T = self.matrix[(0 + i) as usize][(0 + j) as usize];
                    self.matrix[(0 + i) as usize][(0 + j) as usize] =
                        self.matrix[(len - j) as usize][(0 + i) as usize];

                    self.matrix[(len - j) as usize][(0 + i) as usize] =
                        self.matrix[(len - i) as usize][(len - j) as usize];

                    self.matrix[(len - i) as usize][(len - j) as usize] =
                        self.matrix[(0 + j) as usize][(len - i) as usize];

                    self.matrix[(0 + j) as usize][(len - i) as usize] = base
                }
            }
        } else {
            let mut new_matrix: Matrix<T> =
                Matrix::create(self.size_y, self.size_x, &self.matrix[0][0]);
            for i in 0..self.size_x {
                for j in 0..self.size_y {
                    let value = self.matrix[((self.size_y - 1) - j) as usize][i as usize];
                    new_matrix.put((i, j), value)
                }
            }
        }
    }
}

pub struct QRcode {
    pub rs: ReedSolomonData,
    pub size: u8,
    qrcode: Matrix<u8>,
}

impl QRcode {
    pub fn new(data: &str, version: u8, error_correct: ErrorLevel, mask: Mask) -> QRcode {
        let reed_solomon: ReedSolomonData =
            ReedSolomonData::new(data, error_correct, version, mask, ByteNameOfMode::Byte);
        let size = 21 + (reed_solomon.version - 1) * 4;
        let qrcode = QRcode {
            rs: reed_solomon,
            size: size,
            qrcode: Matrix::<u8>::create(size as u32, size as u32, &2),
        };
        return qrcode;
    }
    fn margin(&mut self) {
        let size = (self.size + 6) as u32;
        let mut margin: Matrix<u8> = Matrix::create(size, size, &1);
        margin.put_matrix((3, 3), &self.qrcode);
        self.qrcode = margin;
    }
    pub fn render(&mut self) {
        self.set_patterns();
        self.insert_data();
        self.margin();
    }
    pub fn push_data_strip(&mut self, vec: &mut Vec<u8>, up: bool, point: (u32, u32), swap: u16) {
        let mut y = if up {
            self.qrcode.size_y - point.1 - 1
        } else {
            point.1
        };
        let mut i = 0;
        let mut add = 0;
        let mut index: u16 = swap;
        let len = vec.len();
        while index <= (len + add) as u16 {
            if (up && y < 0) || (!up && y > self.qrcode.size_y - 1) {
                break;
            }
            let p_0 = (self.qrcode.matrix[y as usize].len() as i16 - 2) + (index % 2) as i16
                - point.0 as i16;
            if p_0 < 0 || self.qrcode.matrix[y as usize][p_0 as usize] != 2 {
                // to repair
                add += 1;
                if index % 2 == 0 {
                    if up && y > 0 {
                        y -= 1
                    } else {
                        y += 1
                    }
                }
                index += 1;
                continue;
            }
            if vec.len() == 0 {
                break;
            };
            let mut value = (*vec).pop().unwrap();
            value = if mask(self.rs.mask, p_0 as u16, y as u16) {
                value
            } else {
                if value == 1 {
                    0
                } else {
                    1
                }
            };
            self.qrcode.matrix[y as usize][p_0 as usize] = value;

            if index % 2 == 0 {
                if up && y > 0 {
                    y -= 1
                } else {
                    y += 1
                }
            }
            index += 1;
            i += 1
        }
    }
    pub fn insert_data(&mut self) {
        let mut data_set: Vec<u8> = Vec::new();

        for i in self.rs.bits.chars() {
            data_set.push(i.to_digit(10).unwrap() as u8);
        }
        data_set.reverse();
        let mut col = 0;
        for i in 0..(self.size / 2) {
            self.push_data_strip(
                &mut data_set,
                if i % 2 == 0 { true } else { false },
                (col, 0),
                1,
            );
            col += 2;
            if col == self.size as u32 - 7 || col == self.size as u32 - 6 {
                col += 1
            }
        }
    }
    pub fn create_align(&mut self) {
        let align: &Vec<u32> = &get_alignment(self.rs.version);
        let mut align_matrix: Matrix<u8> = Matrix::create(5, 5, &0);
        let mut align_matrix_border: Matrix<u8> = Matrix::create(3, 3, &1);
        align_matrix_border.put((1, 1), 0);
        align_matrix.put_matrix((1, 1), &align_matrix_border);

        for i in align {
            for j in align {
                if self.qrcode.get((*i, *j)) != 2 {
                    continue;
                }
                self.qrcode.put_matrix((*i - 2, *j - 2), &align_matrix)
            }
        }
    }
    pub fn set_patterns(&mut self) {
        self.create_finder();
        self.black_module();
        if self.rs.version > 2 {
            self.create_align()
        };
        self.create_timing();
        self.create_format_string();
        if self.rs.version >= 7 {
            self.create_version_string()
        }
    }
    pub fn black_module(&mut self) {
        self.qrcode.put((8, self.qrcode.size_y - 8), 0)
    }
    pub fn create_version_string(&mut self) {
        let version: String = self.rs.create_version_string();
        let mut version_vec: Vec<u8> = Vec::new();
        for i in version.chars() {
            version_vec.push(if i.to_digit(10).unwrap() == 1 { 0 } else { 1 });
        }
        let mut version_matrix: Matrix<u8> = Matrix::create(3, 6, &0);
        for i in 0..6 {
            let mut pop_vec: Vec<u8> = Vec::new();
            for j in 0..3 {
                let value = version_vec.pop().unwrap();
                pop_vec.push(value as u8);
            }
            version_matrix.put_vec((0, i), &(pop_vec.to_vec()), true)
        }
        self.qrcode
            .put_matrix((self.size as u32 - 11, 0), &version_matrix);
        version_matrix.transpose();
        self.qrcode
            .put_matrix((0, self.size as u32 - 11), &version_matrix);
    }
    pub fn create_format_string(&mut self) {
        let format: String = self.rs.create_format_string();
        let mut format_vec: Vec<u8> = Vec::new();
        for i in format.chars() {
            format_vec.push(if i.to_digit(10).unwrap() == 1 { 0 } else { 1 });
        }
        let scrap: Vec<u8> = format_vec[(format_vec.len() - 2)..].to_vec();
        for i in 0..2 {
            let point = i % 2 == 0;
            let mut cord: [(u32, u32); 3] = [(0, 8), (7, 8), ((self.size - 8) as u32, 8)];
            if !point {
                format_vec.reverse();
                cord = [(8, 0), (8, 7), (8, (self.size - 7) as u32)];
                format_vec.remove(7);
            }
            self.qrcode
                .put_vec(cord[0], &format_vec[..6].to_vec(), point);
            self.qrcode
                .put_vec(cord[1], &format_vec[6..8].to_vec(), point);
            self.qrcode
                .put_vec(cord[2], &format_vec[7..].to_vec(), point);
        }
    }
    pub fn create_timing(&mut self) {
        let timing: Vec<u8> = (0..(self.size - 14)).map(|x| x % 2).collect();
        self.qrcode.put_vec((6, 6), &timing, true);
        self.qrcode.put_vec((6, 6), &timing, false);
    }
    pub fn create_finder(&mut self) {
        let mut finder: Matrix<u8> = Matrix::create(8, 8, &0);
        let white_finder: Matrix<u8> = Matrix::<u8>::create(5, 5, &1);
        let black_finder = Matrix::<u8>::create(3, 3, &0);
        finder.put_matrix((1, 1), &white_finder);
        finder.put_matrix((2, 2), &black_finder);
        finder.put_vec((7, 0), &[1; 8].to_vec(), false);
        finder.put_vec((0, 7), &[1; 8].to_vec(), true);

        self.qrcode.put_matrix((0, 0), &finder);
        finder.rotate();
        self.qrcode.put_matrix((self.size as u32 - 8, 0), &finder);
        finder.rotate();
        finder.rotate();
        self.qrcode.put_matrix((0, self.size as u32 - 8), &finder);
    }
}
