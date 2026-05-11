export type SoundcloudSettings = {
    profileUrl: string;
    syncInterval: number;
    autoSync: boolean;
}

export type DownloadSettings = {
    outputPath: string;
    maxConcurrent: number;
    namingTemplate: string;
}

export type SystemSettings = {
    limitPerPage: number;
    maxRetries: number;
}

export type SettingsFormModel = {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    system: SystemSettings;
}
