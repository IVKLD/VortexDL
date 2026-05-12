import {HttpClient} from '@angular/common/http';
import {inject, Injectable} from '@angular/core';

export interface GithubApi {
    items: GithubIssue[];
    total_count: number;
}

export interface GithubIssue {
    createdAt: string;
    number: string;
    state: string;
    title: string;
}

@Injectable({providedIn: 'platform'})
export class PlaylistViewService {
    private readonly _httpClient = inject(HttpClient);
}
