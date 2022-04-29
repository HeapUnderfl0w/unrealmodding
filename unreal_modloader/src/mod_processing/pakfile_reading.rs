use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use log::{debug, warn};
use unreal_modintegrator::metadata::{Metadata, SyncMode};
use unreal_pak::PakFile;

use crate::game_mod::{GameMod, GameModVersion, SelectedVersion};
use crate::version::Version;
use crate::AppData;

use super::verify;

#[derive(Debug)]
pub(crate) struct ReadData(String, Metadata);

pub(crate) fn read_pak_files(mod_files: &Vec<PathBuf>) -> HashMap<String, Vec<ReadData>> {
    let mut mods_read: HashMap<String, Vec<ReadData>> = HashMap::new();

    // read metadata
    for file_path in mod_files.iter() {
        let file_result = (|| -> Result<(), Box<dyn Error>> {
            let file = fs::File::open(&file_path)?;
            let mut pak = PakFile::reader(&file);

            pak.load_records()?;

            let record = pak
                .get_record(&String::from("metadata.json"))?
                .data
                .as_ref()
                .unwrap();
            let metadata: Metadata = serde_json::from_slice(&record)?;

            let file_name = file_path.file_name().unwrap().to_str().unwrap().to_owned();

            // check that filename generally matches
            if !verify::verify_mod_file_name(&file_name) {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Mod file {:?} does not match expected format", file_name),
                )));
            }

            let file_name_parts = file_name.split('_').collect::<Vec<&str>>()[0]
                .split("-")
                .collect::<Vec<&str>>();

            // check that mod id in file name matches metadata
            if file_name_parts[1] != metadata.mod_id {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Mod id in file name does not match metadata id: {:?} != {:?}",
                        file_name_parts[1], metadata.mod_id
                    ),
                )));
            }

            // check that version in file name matches metadata
            if file_name_parts[2] != metadata.mod_version {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Version in file name does not match metadata version: {:?} != {:?}",
                        file_name_parts[2], metadata.mod_version
                    ),
                )));
            }

            let mod_id = metadata.mod_id.to_owned();

            if !mods_read.contains_key(&mod_id) {
                mods_read.insert(mod_id.to_owned(), Vec::new());
            }

            mods_read
                .get_mut(&mod_id)
                .unwrap()
                .push(ReadData(file_name, metadata));

            Ok(())
        })();
        match &file_result {
            Ok(_) => {
                debug!(
                    "Successfully read metadata for {:?}",
                    file_path.file_name().unwrap().to_str().unwrap()
                );
            }
            Err(e) => {
                warn!(
                    "Failed to read pak file {:?}, error: {}",
                    file_path.file_name().unwrap().to_str().unwrap(),
                    e
                );
            }
        }
    }

    mods_read
}

pub(crate) fn insert_mods_from_readdata(
    mods_read: &HashMap<String, Vec<ReadData>>,
    data: &mut AppData,
) {
    for (mod_id, mod_files) in mods_read.iter() {
        // check if mod is in global list, if not insert empty
        if !data.game_mods.contains_key(mod_id) {
            let game_mod = GameMod {
                versions: BTreeMap::new(),
                selected_version: SelectedVersion::LatestIndirect(None),
                latest_version: None,

                active: false,

                name: "".to_owned(),
                author: None,
                description: None,
                game_build: None,
                sync: SyncMode::ServerAndClient,
                homepage: None,
                download: None,
                size: 0,
            };

            data.game_mods.insert(mod_id.to_owned(), game_mod);
        }

        // insert metadata
        for read_data in mod_files {
            let version = GameModVersion {
                file_name: read_data.0.clone(),
                downloaded: true,
                download_url: None,
                metadata: Some(read_data.1.clone()),
            };
            let key: Result<Version, _> =
                Version::try_from(&version.metadata.as_ref().unwrap().mod_version);

            if key.is_err() {
                warn!(
                    "Failed to parse version {:?} from metadata for mod {:?}",
                    version.metadata.as_ref().unwrap().mod_version,
                    mod_id
                );

                continue;
            }

            data.game_mods
                .get_mut(mod_id)
                .unwrap()
                .versions
                .insert(key.unwrap(), version);
        }
    }
}
