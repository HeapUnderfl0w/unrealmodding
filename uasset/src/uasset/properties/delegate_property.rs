use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt, Asset}, optional_guid};

#[derive(Hash, PartialEq, Eq)]
pub struct MulticastDelegate {
    number: i32,
    delegate: FName
}

#[derive(Hash, PartialEq, Eq)]
pub struct MulticastDelegateProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    pub value: Vec<MulticastDelegate>
}

impl MulticastDelegate {
    pub fn new(number: i32, delegate: FName) -> Self {
        MulticastDelegate { number, delegate }
    }
}

impl MulticastDelegateProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64, asset: &mut Asset) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let length = cursor.read_i32::<LittleEndian>()?;
        let value = Vec::with_capacity(length as usize);
        for i in 0..length as usize {
            value[i] = MulticastDelegate::new(cursor.read_i32::<LittleEndian>()?, asset.read_fname()?);
        }

        Ok(MulticastDelegateProperty {
            name,
            property_guid,
            value
        })
    }
}