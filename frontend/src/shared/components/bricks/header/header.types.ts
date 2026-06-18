export enum HeaderFeature {
    Search = 'search',
    Sort = 'sort',
    Stats = 'stats'
}

export interface HeaderConfig {
    title: string;
    features: HeaderFeature[];
}

