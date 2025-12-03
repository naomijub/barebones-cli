use std::error::Error;

use self_update::{cargo_crate_version, update::Release};
use semver::Version;

use crate::logger::{log_debug, log_info, log_trace, log_warn};

const DEFAULT_VERSION: Version = Version::new(0, 0, 0);

#[derive(Debug)]
pub struct Updater {
    current_version: String,
    repo_owner: String,
    repo_name: String,
    bin_name: String,
}

impl Updater {
    pub fn new(repo_owner: &str, repo_name: &str, bin_name: &str) -> Self {
        Self {
            current_version: cargo_crate_version!().to_string(),
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            bin_name: bin_name.to_string(),
        }
    }

    /// Checks if there is a newer version available
    pub fn check_for_latest(&self) -> Result<Option<String>, Box<dyn Error>> {
        let mut releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(&self.repo_owner)
            .repo_name(&self.repo_name)
            .build()?
            .fetch()?;
        releases.sort_by(|a, b| {
            Version::parse(&b.version)
                .unwrap_or(DEFAULT_VERSION)
                .cmp(&Version::parse(&a.version).unwrap_or(DEFAULT_VERSION))
        });
        log_debug("Available releases:");
        debug_releases(&releases);

        if let Some(latest) = releases.first() {
            let latest_version = latest.version.trim_start_matches('v');
            let current = semver::Version::parse(&self.current_version)?;
            let latest_semver = semver::Version::parse(latest_version)?;

            if latest_semver > current {
                log_trace(format!("Latest Version: {}", latest_semver));
                return Ok(Some(latest.version.clone()));
            }
        }

        Ok(None)
    }

    /// Perform the update
    pub fn update(&self, verbose: bool) -> Result<Version, Box<dyn Error>> {
        log_debug("Retrieving update");

        let status = self_update::backends::github::Update::configure()
            .repo_owner(&self.repo_owner)
            .repo_name(&self.repo_name)
            .bin_name(&self.bin_name)
            .show_download_progress(true)
            .show_output(verbose)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;

        Ok(Version::parse(status.version())?)
    }

    /// Check and update if newer version exists
    pub fn check_and_update(&self, auto: bool, verbose: bool) -> Result<(), Box<dyn Error>> {
        if let Some(new_version) = self.check_for_latest()? {
            log_warn(format!(
                "New version available: {} (current: {})",
                new_version, self.current_version
            ));

            if auto {
                log_info("Automatically updating...");
                let result = self.update(verbose)?;
                log_info(format!("Update done. Version {}", result));
                log_info("Please restart the application to use the new version.");
            } else {
                log_warn("Run 'barebones-cli update' to install the latest version.");
            }
        };
        Ok(())
    }
}

fn debug_releases(releases: &[Release]) {
    for (version, date, asset) in releases.iter().flat_map(|release| {
        release
            .assets
            .iter()
            .map(|asset| (&release.version, &release.date, asset))
    }) {
        let debugger = format!(
            "{} v{}#{} URL: {}",
            asset.name, version, date, asset.download_url
        );
        log_debug(debugger);
    }
}
