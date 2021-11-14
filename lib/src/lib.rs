// Copyright 2021 Oxide Computer Company

mod sys;

use std::collections::BTreeMap;
use std::fmt::{Display, Error, Formatter};

use num_enum::TryFromPrimitive;

pub use crate::sys::get_devices;

#[derive(TryFromPrimitive)]
#[repr(i32)]
pub enum DiPropType {
    Boolean,
    Int,
    String,
    Byte,
    Unknown,
    UndefIt,
    Int64,
}

#[derive(Debug)]
pub enum DiPropValue {
    Boolean(bool),
    Ints(Vec<i32>),
    Int64s(Vec<i64>),
    Strings(Vec<String>),
}

impl DiPropValue {
    pub fn matches_int(&self, x: i32) -> bool {
        match self {
            Self::Ints(xs) => {
                if xs.len() != 1 {
                    return false;
                }
                xs[0] == x
            }
            _ => false,
        }
    }
}

impl Display for DiPropValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Boolean(x) => write!(f, "{}", x),
            Self::Ints(x) => write!(f, "{:x?}", x),
            Self::Int64s(x) => write!(f, "{:x?}", x),
            Self::Strings(x) => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub props: BTreeMap<String, DiPropValue>,
    pub prom_props: BTreeMap<String, Vec<u8>>,
}

impl DeviceInfo {
    pub fn new() -> DeviceInfo {
        DeviceInfo {
            props: BTreeMap::new(),
            prom_props: BTreeMap::new(),
        }
    }
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
