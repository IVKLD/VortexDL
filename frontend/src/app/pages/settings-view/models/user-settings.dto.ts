import { AdbSettings, DownloadSettings, SoundcloudSettings, NetworkSettings, SystemSettings, WebDavSettings } from '@app/pages/settings-view/models/settings-form.model';

export interface UserSettingsDto {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    adb: AdbSettings;
    network: NetworkSettings;
    system: SystemSettings;
    webdav: WebDavSettings;
}
