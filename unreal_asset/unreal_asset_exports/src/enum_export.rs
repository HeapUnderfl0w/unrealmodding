//! Enum export

use std::collections::HashMap;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use unreal_asset_base::{
    custom_version::FCoreObjectVersion,
    object_version::ObjectVersion,
    reader::{ArchiveReader, ArchiveWriter},
    types::{FName, PackageIndexTrait},
    Error, FNameContainer,
};

use crate::implement_get;
use crate::ExportTrait;
use crate::{BaseExport, NormalExport};

/// Enum cpp form
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum ECppForm {
    /// Regular
    #[default]
    Regular,
    /// Namespaced
    Namespaced,
    /// Enum class
    EnumClass,
}

/// Enum
#[derive(FNameContainer, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct UEnum {
    /// Enum names
    pub names: Vec<(FName, i64)>,
    /// Enum cpp form
    #[container_ignore]
    pub cpp_form: ECppForm,
}

impl UEnum {
    /// Read a `UEnum` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let mut names = Vec::new();

        if asset.get_object_version() < ObjectVersion::VER_UE4_TIGHTLY_PACKED_ENUMS {
            let num_entries = asset.read_i32::<LE>()?;
            for i in 0..num_entries {
                let name = asset.read_fname()?;
                names.push((name, i as i64));
            }
        } else {
            let custom_version = asset.get_custom_version::<FCoreObjectVersion>();
            if custom_version.version < FCoreObjectVersion::EnumProperties as i32 {
                let num_entries = asset.read_i32::<LE>()?;
                for _i in 0..num_entries {
                    let name = asset.read_fname()?;
                    let index = asset.read_u8()?;
                    names.push((name, index as i64));
                }
            } else {
                let num_entries = asset.read_i32::<LE>()?;
                for _i in 0..num_entries {
                    let name = asset.read_fname()?;
                    let index = asset.read_i64::<LE>()?;
                    names.push((name, index));
                }
            }
        }

        let cpp_form = match asset.get_object_version() < ObjectVersion::VER_UE4_ENUM_CLASS_SUPPORT
        {
            true => {
                let is_namespace = asset.read_i32::<LE>()? == 1;
                match is_namespace {
                    true => ECppForm::Namespaced,
                    false => ECppForm::Regular,
                }
            }
            false => asset.read_u8()?.try_into()?,
        };

        Ok(UEnum { names, cpp_form })
    }

    /// Write a `UEnum` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_i32::<LE>(self.names.len() as i32)?;
        if asset.get_object_version() < ObjectVersion::VER_UE4_TIGHTLY_PACKED_ENUMS {
            // todo: a better algorithm?
            let mut names_map = HashMap::with_capacity(self.names.len());
            for (name, index) in &self.names {
                names_map.insert(*index, name.clone());
            }
            for i in 0..names_map.len() {
                if let Some(name) = names_map.get(&(i as i64)) {
                    asset.write_fname(name)?;
                }
            }
        } else if asset.get_custom_version::<FCoreObjectVersion>().version
            < FCoreObjectVersion::EnumProperties as i32
        {
            for (name, index) in &self.names {
                asset.write_fname(name)?;
                asset.write_u8(*index as u8)?;
            }
        } else {
            for (name, index) in &self.names {
                asset.write_fname(name)?;
                asset.write_i64::<LE>(*index)?;
            }
        }

        if asset.get_object_version() < ObjectVersion::VER_UE4_ENUM_CLASS_SUPPORT {
            asset.write_i32::<LE>(match self.cpp_form == ECppForm::Namespaced {
                true => 1,
                false => 0,
            })?;
        } else {
            asset.write_u8(self.cpp_form.into())?;
        }
        Ok(())
    }
}

/// Enum export
#[derive(FNameContainer, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumExport<Index: PackageIndexTrait> {
    /// Base normal export
    pub normal_export: NormalExport<Index>,
    /// Enum value
    pub value: UEnum,
}

implement_get!(EnumExport);

impl<Index: PackageIndexTrait> EnumExport<Index> {
    /// Read an `EnumExport` from an asset
    pub fn from_base<Reader: ArchiveReader<Index>>(
        base: &BaseExport<Index>,
        asset: &mut Reader,
    ) -> Result<Self, Error> {
        let normal_export = NormalExport::from_base(base, asset)?;
        asset.read_i32::<LE>()?;

        let value = UEnum::new(asset)?;
        Ok(EnumExport {
            normal_export,
            value,
        })
    }
}

impl<Index: PackageIndexTrait> ExportTrait<Index> for EnumExport<Index> {
    fn write<Writer: ArchiveWriter<Index>>(&self, asset: &mut Writer) -> Result<(), Error> {
        self.normal_export.write(asset)?;
        asset.write_i32::<LE>(0)?;
        self.value.write(asset)?;
        Ok(())
    }
}