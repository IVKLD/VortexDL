import { DeletedTrack } from './components/deleted-tracks/deleted-tracks.model';

export type LibraryStats = {
    totalTracks: number;
    totalSize: number;
    totalDownloaded: number;
    totalDownloadedBytes: number;
    totalDeleted: number;
    totalDeletedBytes: number;
    recentDeleted: DeletedTrack[];
};
