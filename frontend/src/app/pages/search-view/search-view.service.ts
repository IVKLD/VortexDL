import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { SearchRdo, DownloadDto } from './models/search-view.model';

@Injectable({ providedIn: 'root' })
export class SearchViewService {
    private readonly _http = inject(HttpClient);

    public searchTracks(
        query: string,
        limit = 20,
        offset = 0,
        provider?: string,
        duration?: string
    ): Observable<SearchRdo> {
        const params: Record<string, string | number> = { query, limit, offset };
        if (provider) params['provider'] = provider;
        if (duration && duration !== 'any') params['duration'] = duration;
        return this._http.get<SearchRdo>('/search/tracks', { params });
    }

    public downloadTrack(payload: DownloadDto): Observable<void> {
        return this._http.post<void>('/download', payload);
    }
}
