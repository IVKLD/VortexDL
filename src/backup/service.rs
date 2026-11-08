use std::path::Path;

use crate::{
    backup::{
        BackupAction, BackupBackend, BackupError, BackupSnapshot, BackupTarget,
        backends::{HttpUrlBackend, LocalBackend, WebDavBackend},
        snapshot::SNAPSHOT_VERSION,
    },
    infra::db::{SettingsRepo, TrackCacheRepo},
};

pub struct BackupService<B = ()> {
    backend: B,
}

fn create_temp_backup_file() -> Result<tempfile::NamedTempFile, BackupError> {
    Ok(tempfile::Builder::new()
        .prefix("vortex_backup_")
        .suffix(".json")
        .tempfile()?)
}

impl<B: BackupBackend> BackupService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub async fn execute_action(&self, action: BackupAction) -> Result<(), BackupError> {
        match action {
            BackupAction::Export => self.export().await,
            BackupAction::Import => self.import().await,
        }
    }

    pub async fn export(&self) -> Result<(), BackupError> {
        let temp_file = create_temp_backup_file()?;
        self.do_export(temp_file.path()).await
    }

    pub async fn import(&self) -> Result<(), BackupError> {
        let temp_file = create_temp_backup_file()?;
        self.do_import(temp_file.path()).await
    }

    async fn do_export(&self, tmp: &Path) -> Result<(), BackupError> {
        let snapshot = BackupService::create_snapshot()?;

        let json = serde_json::to_string_pretty(&snapshot)?;
        tokio::fs::write(tmp, json.as_bytes()).await?;
        tracing::debug!("Snapshot written ({} bytes)", json.len());

        self.backend.upload(tmp).await?;
        Ok(())
    }

    async fn do_import(&self, tmp: &Path) -> Result<(), BackupError> {
        self.backend.download(tmp).await?;

        let bytes = tokio::fs::read(tmp).await?;
        tracing::debug!("Read {} bytes from downloaded backup", bytes.len());

        let snapshot: BackupSnapshot = serde_json::from_slice(&bytes)?;
        BackupService::apply_snapshot(snapshot)?;

        Ok(())
    }
}

impl BackupService<()> {
    pub fn create_snapshot() -> Result<BackupSnapshot, BackupError> {
        let mut settings = SettingsRepo::get()?;
        settings.webdav.password.clear();

        let cached = TrackCacheRepo::get_all()?;
        let synced_ids = cached
            .into_iter()
            .filter(|(path, _)| Path::new(path).exists())
            .map(|(_, track)| track.metadata.id)
            .collect();

        Ok(BackupSnapshot {
            version: SNAPSHOT_VERSION,
            settings,
            synced_ids,
        })
    }

    pub fn apply_snapshot(mut snapshot: BackupSnapshot) -> Result<(), BackupError> {
        if snapshot.settings.webdav.password.is_empty() {
            let current = SettingsRepo::get()?;
            snapshot.settings.webdav.password = current.webdav.password;
        }

        SettingsRepo::update(&snapshot.settings)?;
        Ok(())
    }

    pub async fn execute_target(
        target: &BackupTarget,
        action: BackupAction,
    ) -> Result<(), BackupError> {
        match target {
            BackupTarget::Local { path } => {
                BackupService::new(LocalBackend::new(path))
                    .execute_action(action)
                    .await
            }
            BackupTarget::Url { url } => {
                let backend = HttpUrlBackend::new(url.as_str())?;
                BackupService::new(backend).execute_action(action).await
            }
            BackupTarget::WebDav(webdav) => {
                let backend = WebDavBackend::new(
                    webdav.base_url.as_str(),
                    webdav.remote_dir.to_string_lossy(),
                    &webdav.username,
                    &webdav.password,
                )?;
                BackupService::new(backend).execute_action(action).await
            }
        }
    }
}
