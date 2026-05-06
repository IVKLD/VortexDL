export enum HeaderFeature {
    Search = 'search',
    Sort = 'sort',
    Stats = 'stats',
    Download = 'download'
}

export interface HeaderConfig {
    title: string;
    features: HeaderFeature[];
}
