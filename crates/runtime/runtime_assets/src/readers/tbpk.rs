use bevy::prelude::Commands;
use snafu::{ResultExt, Snafu, ensure};
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::pack::HEADER_LEN;
use crate::{ManifestLoadError, ManifestSource, RuntimePackConfig};

/// a pack, or a dev directory standing in for one, that couldn't be read
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum PackReadError {
    #[snafu(display(
        "`{}` is a directory, but reading dev directories as packs is off (`allow_dir_read` is false, run with `--dev` to enable it)",
        path.display()
    ))]
    DirReadNotAllowed { path: PathBuf },

    #[snafu(display("failed to load the manifest of dev directory `{}`", path.display()))]
    DevManifest {
        path: PathBuf,
        #[snafu(source(from(ManifestLoadError, Box::new)))]
        source: Box<ManifestLoadError>,
    },

    #[snafu(display("failed to open pack file `{}`", path.display()))]
    Open {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to read the metadata of pack file `{}`", path.display()))]
    Metadata {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "pack file `{}` is {len} bytes, too short to hold the {HEADER_LEN} byte common header, it is truncated or not a pack",
        path.display()
    ))]
    TooShort { path: PathBuf, len: u64 },
}

pub struct PackReader {}

impl PackReader {
    #[tracing::instrument(skip_all, fields(pack = %path.display()))]
    pub fn read_pack(
        commands: &mut Commands,
        path: &Path,
        config: &RuntimePackConfig,
    ) -> Result<(), PackReadError> {
        info!("Reading pack at path {}", path.display());

        if path.is_dir() {
            ensure!(
                *config.get_allow_dir_read(),
                DirReadNotAllowedSnafu { path }
            );

            info!("Path is directory, continuing as a developer package. Assuming newest version");

            let _manifest_source = ManifestSource::from_path(&path.join("manifest.toml"))
                .context(DevManifestSnafu { path })?;

            return Ok(());
        }

        info!("path is not directory, assuming is of type file");

        let file = File::open(path).context(OpenSnafu { path })?;
        let len = file.metadata().context(MetadataSnafu { path })?.len();
        ensure!(len >= HEADER_LEN as u64, TooShortSnafu { path, len });
        info!("Pack file is {len} bytes");

        Ok(())
    }
}
