import {HttpClient} from '@angular/common/http';
import {inject, Injectable} from '@angular/core';
import {SortDirection} from '@angular/material/sort';
import {Observable} from 'rxjs';

export interface GithubApi {
    items: GithubIssue[];
    total_count: number;
}

export interface GithubIssue {
    created_at: string;
    number: string;
    state: string;
    title: string;
}

@Injectable({providedIn: "platform"})
export class PlaylistViewService {
    private readonly _httpClient = inject(HttpClient);

    getRepoIssues(sort: string, order: SortDirection, page: number): Observable<GithubApi> {
        const href = 'https://api.github.com/search/issues';
        const requestUrl = `${href}?q=repo:angular/components&sort=${sort}&order=${order}&page=${
            page + 1
        }`;

        return this._httpClient.get<GithubApi>(requestUrl);
    }
}