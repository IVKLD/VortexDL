export type SoundcloudSettings = {
    profileUrl: string;
    syncInterval: number;
    autoSync: boolean;
    cachedClientId?: string;
}

export enum SyncMode {
    Silent = 'silent',
    Full = 'full',
    Archive = 'archive'
}

export type DownloadSettings = {
    outputPath: string;
    maxConcurrent: number;
    namingTemplate: string;
    syncMode: SyncMode;
}

export type SystemSettings = {
    limitPerPage: number;
    maxRetries: number;
}

export type NetworkSettings = {
    useProxy: boolean;
    proxyUrl: string;
    fallbackProxies: string[];
}

export type AdbDeviceSettings = {
    deviceId: string;
    remoteMusicDir: string;
    enabled: boolean;
}

export type AdbSettings = {
    enabled: boolean;
    autoSync: boolean;
    devices: AdbDeviceSettings[];
}

export type SettingsFormModel = {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    system: SystemSettings;
    network: NetworkSettings;
    adb: AdbSettings;
}
