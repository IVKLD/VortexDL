export enum StorageType {
    Internal = 'Internal',
    SdCard = 'SDCard',
}

export type StorageInfo = {
    name: string;
    path: string;
    storageType: StorageType;
}
