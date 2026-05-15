import {AdbSettings, DownloadSettings, SoundcloudSettings} from "@app/pages/settings-view/models/settings-form.model";

export interface UserSettingsRdo {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    adb: AdbSettings;
    limitPerPage: number,
    maxRetries: number
}
