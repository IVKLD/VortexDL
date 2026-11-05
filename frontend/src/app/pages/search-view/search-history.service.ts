import { Injectable } from '@angular/core';
import { BaseStorageHistoryService } from '@shared/utils/search-history.utils';

@Injectable({ providedIn: 'root' })
export class SearchHistoryService extends BaseStorageHistoryService {
    constructor() {
        super('search_history', 10);
    }
}
