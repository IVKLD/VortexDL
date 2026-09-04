export type SoundcloudSettings = {
    profileUrl: string;
    syncInterval: number;
    autoSync: boolean;
    cachedClientId?: string;
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

export type WebDavSettings = {
    baseUrl: string;
    remoteDir: string;
    username: string;
    password: string;
}

export type SettingsFormModel = {
    soundcloud: SoundcloudSettings;
    downloads: DownloadSettings;
    system: SystemSettings;
    network: NetworkSettings;
    adb: AdbSettings;
    webdav: WebDavSettings;
}
