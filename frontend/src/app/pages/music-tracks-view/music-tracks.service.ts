import { HttpClient, HttpParams } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { MusicTracks } from '@shared/models/music-track.model';
import { Observable } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class MusicTracksService {
    private readonly _http = inject(HttpClient);

    public getAll(sort = 'date', order = 'desc', limit?: number): Observable<MusicTracks> {
        let params = new HttpParams()
            .set('sort', sort.toLowerCase())
            .set('order', order.toLowerCase());

        if (limit) {
            params = params.set('limit', limit.toString());
        }

        return this._http.get<MusicTracks>('/downloads', { params });
    }

    public indexing(): Observable<void> {
        return this._http.post<void>('/library/reindex', null);
    }

    public delete(ids: number[]): Observable<void> {
        return this._http.delete<void>('/downloads', { body: ids });
    }

    public download(url: string): Observable<void> {
        return this._http.post<void>('/download', { url });
    }
}
