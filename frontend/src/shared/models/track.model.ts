export enum AudioFormat {
    MP3 = 'mp3',
    FLAC = 'flac',
    WAV = 'wav',
    UNKNOWN = 'unknown',
}

export interface Track {
    id: number;
    artist: string;
    title: string;
    format: AudioFormat;
    artworkUrl: string | null;
    sourceUrl: string | null;
    createdAt: number;
    size: number;
    position: number;
}

export type Tracks = Track[];
