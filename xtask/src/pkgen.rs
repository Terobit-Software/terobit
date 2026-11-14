//! `cargo xtask pkgen`: builds `.tbpk` packs from a dev directory. it stands in for the
//! editor's publish step until the editor exists

use std::fmt;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use runtime_assets::pack::Target;
use runtime_assets::pack::writers::{PackWriter, PackWriterError};
use runtime_consts::pack::ZSTD_LEVEL;
use runtime_consts::paths::TBPK_GENERATOR_OUT;
use snafu::{Report, ResultExt, Snafu, ensure};
use tracing::{error, info, warn};

#[derive(Debug, Args)]
pub struct PkgenArgs {
    /// The dev directory's manifest.toml, asset paths are relative to its folder
    #[arg(long)]
    pub manifest: PathBuf,

    /// Folder the packs are written to, created if missing
    #[arg(long, default_value = TBPK_GENERATOR_OUT)]
    pub out: PathBuf,

    /// Which packs to build
    #[arg(long, value_enum, default_value_t = TargetArg::Both)]
    pub target: TargetArg,

    /// Zstd level chunks are compressed at, or `none` to store them uncompressed. Any other
    /// level than the default changes chunk bytes and hashes, so only use it to test speed and size
    #[arg(long, value_parser = parse_zstd_level, default_value_t = ZstdLevel(Some(ZSTD_LEVEL)))]
    pub zstd_level: ZstdLevel,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TargetArg {
    Client,
    Server,
    Both,
}

impl TargetArg {
    fn targets(self) -> &'static [Target] {
        match self {
            TargetArg::Client => &[Target::Client],
            TargetArg::Server => &[Target::Server],
            TargetArg::Both => &[Target::Client, Target::Server],
        }
    }
}

/// `--zstd-level`, `None` is no compression
#[derive(Clone, Copy, Debug)]
pub struct ZstdLevel(pub Option<i32>);

impl fmt::Display for ZstdLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(level) => write!(f, "{level}"),
            None => f.write_str("none"),
        }
    }
}

fn parse_zstd_level(value: &str) -> Result<ZstdLevel, ParseIntError> {
    if value == "none" {
        return Ok(ZstdLevel(None));
    }
    value.parse().map(|level| ZstdLevel(Some(level)))
}

/// pack generation stopped or one of the targets failed
#[derive(Debug, Snafu)]
pub enum PkgenError {
    #[snafu(display("failed to set up the pack writer"))]
    SetupWriter {
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to load manifest `{}`", path.display()))]
    LoadManifest {
        path: PathBuf,
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to create the output folder `{}`", path.display()))]
    CreateOutDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to name the packs from the loaded manifest"))]
    NamePacks {
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to build the {target:?} pack"))]
    BuildPack {
        target: Target,
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display("failed to write the {target:?} pack to `{}`", path.display()))]
    WritePack {
        target: Target,
        path: PathBuf,
        #[snafu(source(from(PackWriterError, Box::new)))]
        source: Box<PackWriterError>,
    },

    #[snafu(display(
        "{} of {total} targets failed ({failed:?}), see the errors above for why",
        failed.len()
    ))]
    TargetsFailed { failed: Vec<Target>, total: usize },
}

/// a failed target is logged and doesn't stop the other, the error only says which failed
#[tracing::instrument(skip_all, fields(manifest = %args.manifest.display()))]
pub fn run(args: &PkgenArgs) -> Result<(), PkgenError> {
    let targets = args.target.targets();
    info!("Output folder: {}", args.out.display());
    info!("Targets: {targets:?}");
    info!("Zstd level: {}", args.zstd_level);
    let start = std::time::Instant::now();

    // TODO: pass a PackSigner with `.with_signer()` once keys can be loaded,
    // until then packs are written unsigned
    warn!("No signing key loaded, writing unsigned packs");
    let mut writer = PackWriter::new()
        .with_zstd_level(args.zstd_level.0)
        .context(SetupWriterSnafu)?;
    writer
        .load_manifest(args.manifest.clone())
        .context(LoadManifestSnafu {
            path: &args.manifest,
        })?;

    std::fs::create_dir_all(&args.out).context(CreateOutDirSnafu { path: &args.out })?;

    // `<realm_id>-<pack_version>.<target>.tbpk`, as suggested in the tbpk doc
    let source = writer.get_manifest_source().context(NamePacksSnafu)?;
    let stem = format!("{}-{}", source.get_realm_id(), source.get_pack_version());
    info!("Packs will be named {stem}.<target>.tbpk");

    let mut failed = Vec::new();
    for &target in targets {
        let target_start = std::time::Instant::now();
        info!("Generating {target:?} pack");
        let path = args.out.join(format!("{stem}.{}.tbpk", side_name(target)));
        if let Err(e) = write_pack(&mut writer, target, &path) {
            error!(
                "Skipping the {target:?} pack after {:?}: {}",
                target_start.elapsed(),
                Report::from_error(&e)
            );
            failed.push(target);
            continue;
        }
        info!(
            "Generated {target:?} pack at {} in {:?}",
            path.display(),
            target_start.elapsed()
        );
    }

    ensure!(
        failed.is_empty(),
        TargetsFailedSnafu {
            failed,
            total: targets.len()
        }
    );
    info!("Generated all packs in {:?}", start.elapsed());
    Ok(())
}

fn write_pack(writer: &mut PackWriter, target: Target, path: &Path) -> Result<(), PkgenError> {
    writer
        .build_tbpk(target)
        .context(BuildPackSnafu { target })?;
    writer
        .write_tbpk_out(path)
        .context(WritePackSnafu { target, path })
}

fn side_name(target: Target) -> &'static str {
    match target {
        Target::Client => "client",
        Target::Server => "server",
    }
}

#[cfg(test)]
mod fn_tests {
    use clap::Parser;

    use super::*;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        args: PkgenArgs,
    }

    fn parse(args: &[&str]) -> Option<PkgenArgs> {
        TestCli::try_parse_from(std::iter::once("pkgen").chain(args.iter().copied()))
            .ok()
            .map(|cli| cli.args)
    }

    #[test]
    fn defaults() {
        let args = parse(&["--manifest", "m.toml"]).unwrap();
        assert_eq!(args.out, Path::new(TBPK_GENERATOR_OUT));
        assert!(matches!(args.target, TargetArg::Both));
        assert_eq!(args.zstd_level.0, Some(ZSTD_LEVEL));
    }

    #[test]
    fn zstd_level_none_and_number() {
        let args = parse(&["--manifest", "m", "--zstd-level", "none"]).unwrap();
        assert_eq!(args.zstd_level.0, None);
        let args = parse(&["--manifest", "m", "--zstd-level", "19"]).unwrap();
        assert_eq!(args.zstd_level.0, Some(19));
    }

    #[test]
    fn rejects_bad_flags() {
        assert!(parse(&[]).is_none());
        assert!(parse(&["--manifest", "m", "--target", "both2"]).is_none());
        assert!(parse(&["--manifest", "m", "--zstd-level", "max"]).is_none());
    }
}
