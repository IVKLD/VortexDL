import {AdbSettings, DownloadSettings, SoundcloudSettings, NetworkSettings} from "@app/pages/settings-view/models/settings-form.model";

export interface UserSettingsRdo {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    adb: AdbSettings;
    network: NetworkSettings;
    limitPerPage: number,
    maxRetries: number
}
