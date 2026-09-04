export enum SyncAction {
    Export = 'export',
    Import = 'import',
}

export enum SyncProviderType {
    WebDav = 'webDav',
    Local = 'local',
    Url = 'url',
}

export type WebDavProviderPayload = {
    baseUrl: string;
    remoteDir: string;
    username: string;
    password: string;
};

export type LocalProviderPayload = {
    path: string;
};

export type UrlProviderPayload = {
    url: string;
};

export type SyncProviderPayload =
    | { webDav: WebDavProviderPayload }
    | { local: LocalProviderPayload }
    | { url: UrlProviderPayload };

export type SyncSnapshotResponse = {
    syncRecordsCount: number;
};

