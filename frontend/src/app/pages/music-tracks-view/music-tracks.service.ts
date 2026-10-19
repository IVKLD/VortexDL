import {HttpClient, HttpParams} from '@angular/common/http';
import {inject, Injectable, signal} from '@angular/core';
import {MusicTracks} from '@shared/models/music-track.model';
import {finalize} from 'rxjs';

@Injectable({ providedIn: 'root' })
export class MusicTracksService {
    private readonly _http = inject(HttpClient);
    public readonly isLoading = signal<boolean>(true);

    public getAll(sort = 'date', order = 'desc', limit?: number) {
        this.isLoading.set(true);
        let params = new HttpParams()
            .set('sort', sort.toLowerCase())
            .set('order', order.toLowerCase());

        if (limit) {
            params = params.set('limit', limit.toString());
        }

        return this._http.get<MusicTracks>('/downloads', { params }).pipe(
            finalize(() => this.isLoading.set(false))
        );
    }

    public indexing() {
        return this._http.post('/library/reindex', null);
    }

    public delete(ids: number[]) {
        return this._http.delete('/downloads', { body: ids });
    }

    public download(url: string) {
        return this._http.post<void>('/download', { url });
    }
}

