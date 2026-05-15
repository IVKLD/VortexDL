import {AdbSettings, DownloadSettings, SoundcloudSettings} from "@app/pages/settings-view/models/settings-form.model";

export interface UserSettingsDto {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    adb: AdbSettings;
    limitPerPage: number;
    maxRetries: number;
}
