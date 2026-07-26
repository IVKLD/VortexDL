import { PlayableTrack } from '@shared/models/music-track.model';

export interface SearchTrackItemRdo extends PlayableTrack {
    duration: number | null;
    playbackCount: number | null;
    genre: string | null;
}

export interface SearchResponseRdo {
    tracks: SearchTrackItemRdo[];
    hasMore: boolean;
}

export interface StreamUrlResponseRdo {
    url: string;
}

export interface DownloadRequestDto {
    url: string;
}
