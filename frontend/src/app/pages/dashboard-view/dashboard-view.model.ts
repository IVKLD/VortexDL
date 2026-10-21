import {AudioFormat} from '@shared/models/music-track.model';

export interface DeletedTrack {
    id: number;
    title: string;
    artist: string;
    format: string;
    size: number;
    deletedAt: number;
}

export interface LibraryStats {
    totalTracks: number;
    totalSize: number;
    totalDownloaded: number;
    totalDownloadedBytes: number;
    totalDeleted: number;
    totalDeletedBytes: number;
    recentDeleted: DeletedTrack[];
}

export interface ActivityDay {
    label: string;
    downloadCount: number;
    deleteCount: number;
    downloadHeightPercent: number;
    deleteHeightPercent: number;
}

export interface FormatItem {
    format: AudioFormat;
    count: number;
    percentage: number;
    color: string;
    sizeBytes?: number;
}
