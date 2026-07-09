export interface SearchTrackItemRdo {
    id: number;
    title: string;
    artist: string;
    artworkUrl: string | null;
    duration: number | null;
    playbackCount: number | null;
    permalinkUrl: string | null;
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
