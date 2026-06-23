import {HttpClient, HttpParams} from '@angular/common/http';
import {inject, Injectable} from '@angular/core';
import {MusicTracks} from '@shared/models/music-track.model';

@Injectable({ providedIn: 'root' })
export class MusicTracksViewService {
    private readonly _http = inject(HttpClient);

    public getAll(sort = 'date', order = 'desc', limit?: number) {
        let params = new HttpParams()
            .set('sort', sort.toLowerCase())
            .set('order', order.toLowerCase());

        if (limit) {
            params = params.set('limit', limit.toString());
        }

        return this._http.get<MusicTracks>('/downloads', { params });
    }

    public indexing() {
        return this._http.post('/library/reindex', {});
    }

    public delete(id: number) {
        return this._http.delete(`/downloads/${id}`);
    }

    public download(url: string) {
        return this._http.post('/download', { url });
    }
}

