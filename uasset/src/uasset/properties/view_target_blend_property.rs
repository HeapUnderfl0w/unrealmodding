use std::io::{Cursor, Error, ErrorKind};

use byteorder::{LittleEndian, ReadBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use ordered_float::OrderedFloat;

use crate::{uasset::{unreal_types::{Guid, FName}, cursor_ext::CursorExt}, optional_guid};

#[derive(IntoPrimitive, TryFromPrimitive, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ViewTargetBlendFunction
{
    /** Camera does a simple linear interpolation. */
    VTBlend_Linear,
    /** Camera has a slight ease in and ease out, but amount of ease cannot be tweaked. */
    VTBlend_Cubic,
    /** Camera immediately accelerates, but smoothly decelerates into the target.  Ease amount controlled by BlendExp. */
    VTBlend_EaseIn,
    /** Camera smoothly accelerates, but does not decelerate into the target.  Ease amount controlled by BlendExp. */
    VTBlend_EaseOut,
    /** Camera smoothly accelerates and decelerates.  Ease amount controlled by BlendExp. */
    VTBlend_EaseInOut,
    VTBlend_MAX,
}

#[derive(Hash, PartialEq, Eq)]
pub struct ViewTargetBlendParamsProperty {
    pub name: FName,
    pub property_guid: Option<Guid>,
    
    pub blend_time: OrderedFloat<f32>,
    pub blend_function: ViewTargetBlendFunction,
    pub blend_exp: OrderedFloat<f32>,
    pub lock_outgoing: bool
}

impl ViewTargetBlendParamsProperty {
    pub fn new(name: FName, cursor: &mut Cursor<Vec<u8>>, include_header: bool, length: i64) -> Result<Self, Error> {
        let property_guid = optional_guid!(cursor, include_header);

        let blend_time = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let blend_function = ViewTargetBlendFunction::try_from(cursor.read_u8()?).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        let blend_exp = OrderedFloat(cursor.read_f32::<LittleEndian>()?);
        let lock_outgoing = cursor.read_i32::<LittleEndian>()? != 0;

        Ok(ViewTargetBlendParamsProperty {
            name,
            property_guid,
            blend_time,
            blend_function,
            blend_exp,
            lock_outgoing
        })
    }
}