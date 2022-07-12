#[path = "config.rs"]
mod config;
use config::EntityPolynomial;
use std::vec::Vec;
use std::{fmt, vec};

pub struct Polynomial {
    pub data: Vec<EntityPolynomial>,
}

impl fmt::Debug for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        for _polynomial in &self.data {
            println!(
                "{:?}",
                format!(
                    "|x:{}; alpha:{}; value:{}|",
                    _polynomial.0, _polynomial.1, _polynomial.2
                )
            )
        }
        write!(f, "")
    }
}
pub const BASE_POLYNOMIAL: fn(i16) -> Polynomial = |x| Polynomial::new(vec![(1, 0, 0), (0, x, 0)]);

impl Polynomial {
    pub fn new(value: Vec<EntityPolynomial>) -> Polynomial {
        Polynomial { data: value }
    }
    pub fn new_default() -> Polynomial {
        Polynomial { data: vec![] }
    }
    pub fn find_polynomial(&mut self, target: i16) -> Option<&mut EntityPolynomial> {
        self.data.iter_mut().find(|(x, _a, _v)| *x == target)
    }
    pub fn push(&mut self, value: EntityPolynomial) {
        self.data.push(value);
    }
    pub fn multiply_by_poly(&self, by_polynomial: Polynomial) -> Polynomial {
        let zipped_polynomial: Vec<Vec<EntityPolynomial>> = self
            .data
            .iter()
            .map(|&(v, d, s)| {
                by_polynomial
                    .data
                    .iter()
                    .map(|&(v_by, d_by, _s_by)| (v + v_by, d + d_by, s))
                    .collect::<Vec<EntityPolynomial>>()
            })
            .collect();
        //
        let mut polynomial_combined = Polynomial::new_default();
        let mut _polynomial_map = Polynomial::new_default();
        //
        polynomial_combined.data = zipped_polynomial.concat();
        for polynomial in polynomial_combined.data.into_iter() {
            let mut operand_poly: EntityPolynomial = (polynomial.0, 0, polynomial.2);

            if let Some(val) = _polynomial_map.find_polynomial(polynomial.0) {
                val.1 = val.1 ^ (exponent_galois(polynomial.1 as u32) as i16);
                continue;
            };

            operand_poly.1 = 0 ^ (exponent_galois(polynomial.1 as u32) as i16);
            _polynomial_map.push(operand_poly)
        }
        _polynomial_map.data.reverse();
        let mut _polynomial_map = _polynomial_map
            .data
            .iter()
            .map(|&(x, a, v)| (x, reverse_exponent_galois(a as u32) as i16, v))
            .collect::<Vec<EntityPolynomial>>();

        _polynomial_map.sort_by(|a, b| b.cmp(a));
        return Polynomial::new(_polynomial_map);
    }
    pub fn multiply_by_exp(&self, by_polynomial: Vec<EntityPolynomial>) -> Polynomial {
        let ziped_polynomial: Vec<Vec<EntityPolynomial>> = self
            .data
            .iter()
            .map(|&(v, d, s)| {
                by_polynomial
                    .iter()
                    .map(|&(v_by, d_by, _s_by)| (v + v_by, d + d_by, s))
                    .collect::<Vec<EntityPolynomial>>()
            })
            .collect();
        //
        let mut polynomial_combined = Polynomial::new_default();
        let mut _polynomial_map = Polynomial::new_default();
        //
        polynomial_combined.data = ziped_polynomial.concat();
        for polynomial in polynomial_combined.data.into_iter() {
            let mut operand_poly: EntityPolynomial = (polynomial.0, 0, polynomial.2);

            if let Some(val) = _polynomial_map.find_polynomial(polynomial.0) {
                val.1 = val.1 ^ (exponent_galois(polynomial.1 as u32) as i16);
                continue;
            };

            operand_poly.1 = 0 ^ (exponent_galois(polynomial.1 as u32) as i16);
            _polynomial_map.push(operand_poly)
        }
        _polynomial_map.data.reverse();
        let mut _polynomial_map = _polynomial_map
            .data
            .iter()
            .map(|&(x, a, v)| (x, reverse_exponent_galois(a as u32) as i16, v))
            .collect::<Vec<EntityPolynomial>>();

        _polynomial_map.sort_by(|a, b| b.cmp(a));
        return Polynomial::new(_polynomial_map);
    }
}

fn reduce_galois_operator(operator: &mut u32) {
    if *operator >= 255 {
        while *operator > 255 {
            *operator %= 255
        }
    }
}
fn decrease_polynomial(polynomial_generator: Polynomial) -> Polynomial {
    polynomial_generator.multiply_by_poly(Polynomial::new(vec![(-1, 0, 0)]))
}

pub fn generator_polynomial(number: i32) -> Polynomial {
    let mut tmp_polynomial: Polynomial = BASE_POLYNOMIAL(0);
    for i in 1..number {
        tmp_polynomial = tmp_polynomial.multiply_by_poly(BASE_POLYNOMIAL(i as i16));
    }
    return tmp_polynomial;
}
pub fn div_polynomial(dec_words: &mut Vec<i16>, error_correction: i16) -> Polynomial {
    let mut polynomial_message: Vec<EntityPolynomial> =
        generator_polynomial((dec_words.len() as i32) - 1)
            .data
            .iter_mut()
            .enumerate()
            .map(|(i, exponent)| {
                exponent.0 += error_correction;
                //println!("{:?}", i);
                exponent.2 = dec_words[i];
                *exponent
            })
            .collect();
    let exponent_diff: i16 = polynomial_message[0].0 - error_correction;

    let polynomial_generator_data: Vec<EntityPolynomial> =
        generator_polynomial(error_correction as i32)
            .data
            .iter_mut()
            .map(|x| {
                x.0 = x.0 + exponent_diff;
                *x
            })
            .collect();
    let mut polynomial_generator = Polynomial::new(polynomial_generator_data);
    let mut buffer = 0;
    let mut u = 0;
    while u < (exponent_diff + buffer + (if dec_words.len() < 19 { 1 } else { 0 })) {
        u += 1;
        // MULTI
        let lead_exponent = reverse_exponent_galois(polynomial_message[0].2 as u32) as i16;
        let mut tmp_polynomial_generator: Vec<EntityPolynomial> = polynomial_generator
            .multiply_by_exp(vec![(0, lead_exponent, 0)])
            .data
            .iter_mut()
            .map(|x| {
                x.2 = exponent_galois(x.1 as u32) as i16;
                *x
            })
            .collect();

        //XOR
        let mapping = |base: &mut Vec<EntityPolynomial>,
                       side: &dyn Fn(i16, usize) -> i16|
         -> Vec<EntityPolynomial> {
            base.iter_mut()
                .enumerate()
                .map(|(i, x)| {
                    x.2 = side(x.2, i);
                    *x
                })
                .collect()
        };
        if polynomial_message.len() == tmp_polynomial_generator.len() {
            buffer += 1;
        }
        let by_right: &dyn Fn(i16, usize) -> i16 = &|item, index| {
            let mut value: i16 = 0;
            if tmp_polynomial_generator.get(index).is_some() {
                value = tmp_polynomial_generator.get(index).unwrap().2
            }
            item ^ value as i16
        };
        let by_left: &dyn Fn(i16, usize) -> i16 = &|item, index| {
            let mut value: i16 = 0;
            if polynomial_message.get(index).is_some() {
                value = polynomial_message.get(index).unwrap().2
            }
            value ^ item as i16
        };
        // fn x(item: i16, index: usize) -> i16 {
        //     (item ^ tmp_polynomial_generator[index].2) as i16
        // };
        if polynomial_message.len() > tmp_polynomial_generator.len() {
            polynomial_message = mapping(&mut polynomial_message, by_right)
        } else {
            polynomial_message = mapping(&mut tmp_polynomial_generator, by_left)
        }
        let mut x = 0;
        while polynomial_message[0].2 == 0 {
            polynomial_message.remove(0);
            x += 1;
        }
        if x > 1 {
            buffer -= 1;
            polynomial_generator = decrease_polynomial(polynomial_generator);
        }
        polynomial_generator = decrease_polynomial(polynomial_generator);
    }
    return Polynomial::new(polynomial_message);
}
pub fn exponent_galois(mut exponent: u32) -> u32 {
    reduce_galois_operator(&mut exponent);
    if exponent == 8 {
        return u32::pow(2, exponent) ^ config::MODULO_BYTE_WISE;
    } else if exponent > 8 {
        let prev_power = exponent_galois(exponent - 1) * 2;
        return if prev_power >= 255 {
            prev_power ^ config::MODULO_BYTE_WISE
        } else {
            prev_power
        };
    }
    u32::pow(2, exponent)
}
pub fn reverse_exponent_galois(mut target: u32) -> u32 {
    reduce_galois_operator(&mut target);
    for i in 0u32..255 {
        if target == exponent_galois(i) {
            return i;
        }
    }
    return 0;
}
