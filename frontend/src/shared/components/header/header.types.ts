export enum HeaderFeature {
    Search = 'search',
    Sort = 'sort',
    Stats = 'stats',
    Download = 'download',
    Sync = 'sync',
}

export interface HeaderConfig {
    title: string;
    features: HeaderFeature[];
}
