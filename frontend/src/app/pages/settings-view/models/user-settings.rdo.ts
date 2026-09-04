import { AdbSettings, DownloadSettings, SoundcloudSettings, NetworkSettings, SystemSettings, WebDavSettings } from '@app/pages/settings-view/models/settings-form.model';

export interface UserSettingsRdo {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    adb: AdbSettings;
    network: NetworkSettings;
    system: SystemSettings;
    webdav: WebDavSettings;
}
