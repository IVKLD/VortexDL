import {HttpClient} from '@angular/common/http';
import {inject, Injectable} from '@angular/core';

export type Track = {
    id: number;
    filename: string;
    album: string;
    format: 'wav' | 'flac' | 'mp3';
    artwork_url?: string;
    source_url?: string;
    created_at: number;
}

export type Tracks = Track[]

@Injectable({providedIn: "root"})
export class MusicTracksViewService {
    private readonly _httpClient = inject(HttpClient);

    public getAll() {
        return this._httpClient.get<Tracks>('/downloads');
    }

    public delete(id: number) {
        return this._httpClient.delete('/downloads', {params: {id}});
    }

    public download(url: string) {
        return this._httpClient.post('/download', {url});
    }
}