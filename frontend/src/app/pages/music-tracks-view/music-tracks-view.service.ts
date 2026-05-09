import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';

import { TrackExtension } from '../../models/track.model';

export interface Track {
    id: number;
    filename: string;
    album: string;
    format: TrackExtension;
    artwork_url?: string;
    source_url?: string;
    created_at: number;
}

export type Tracks = Track[];

@Injectable({ providedIn: 'root' })
export class MusicTracksViewService {
    private readonly _http = inject(HttpClient);

    public getAll() {
        return this._http.get<Tracks>('/downloads');
    }

    public delete(id: number) {
        return this._http.delete(`/downloads/${id}`);
    }

    public download(url: string) {
        return this._http.post('/download', { url });
    }
}
