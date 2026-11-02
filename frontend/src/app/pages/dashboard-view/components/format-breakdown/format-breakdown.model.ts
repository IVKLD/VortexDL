import { AudioFormat } from '@shared/models/music-track.model';

export type FormatItem = {
    format: AudioFormat;
    count: number;
    percentage: number;
    color: string;
    sizeBytes?: number;
};
