import {DownloadSettings, SoundcloudSettings} from "@app/pages/settings/models/settings-form.model";

export interface UserSettingsRdo {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    limitPerPage: number,
    maxRetries: number
}
