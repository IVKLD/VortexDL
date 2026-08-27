import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { SearchResponseRdo, StreamUrlResponseRdo, DownloadRequestDto } from './models/search-view.model';

@Injectable({ providedIn: 'root' })
export class SearchViewService {
    private readonly _http = inject(HttpClient);

    public searchTracks(
        query: string,
        limit = 20,
        offset = 0,
        provider?: string,
        duration?: string
    ): Observable<SearchResponseRdo> {
        const params: Record<string, string | number> = { query, limit, offset };
        if (provider) params['provider'] = provider;
        if (duration && duration !== 'any') params['duration'] = duration;
        return this._http.get<SearchResponseRdo>('/search/tracks', { params });
    }

    public downloadTrack(payload: DownloadRequestDto): Observable<void> {
        return this._http.post<void>('/download', payload);
    }

    public getStreamUrl(trackId: number, permalinkUrl?: string): Observable<StreamUrlResponseRdo> {
        const params: Record<string, string> = {};
        if (permalinkUrl) {
            params['url'] = permalinkUrl;
        }
        return this._http.get<StreamUrlResponseRdo>(`/search/tracks/${trackId}/stream`, { params });
    }
}
