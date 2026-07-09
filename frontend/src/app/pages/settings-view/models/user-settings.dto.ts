import {AdbSettings, DownloadSettings, SoundcloudSettings, NetworkSettings, SystemSettings} from "@app/pages/settings-view/models/settings-form.model";

export interface UserSettingsDto {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    adb: AdbSettings;
    network: NetworkSettings;
    system: SystemSettings;
}
