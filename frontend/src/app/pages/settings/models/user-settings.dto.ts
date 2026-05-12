import {DownloadSettings, SoundcloudSettings} from "@app/pages/settings/models/settings-form.model";

export interface UserSettingsDto {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    limitPerPage: number;
    maxRetries: number;
}
