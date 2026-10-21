import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { LibraryStats } from './dashboard-view.model';

@Injectable({ providedIn: 'root' })
export class DashboardViewService {
    private readonly _http = inject(HttpClient);

    public getStats(): Observable<LibraryStats> {
        return this._http.get<LibraryStats>('/downloads/stats');
    }

    public clearDeletedHistory(): Observable<void> {
        return this._http.delete<void>('/downloads/stats');
    }
}
