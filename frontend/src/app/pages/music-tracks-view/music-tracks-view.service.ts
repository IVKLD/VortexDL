import {HttpClient} from '@angular/common/http';
import {inject, Injectable} from '@angular/core';
import {Tracks} from '@shared/models/track.model';

@Injectable({providedIn: 'root'})
export class MusicTracksViewService {
    private readonly _http = inject(HttpClient);

    public getAll() {
        return this._http.get<Tracks>('/downloads');
    }

    public index() {
        return this._http.get('/downloads/indexing_tracks');
    }

    public delete(id: number) {
        return this._http.delete(`/downloads/${id}`);
    }

    public download(url: string) {
        return this._http.post('/download', {url});
    }
}

