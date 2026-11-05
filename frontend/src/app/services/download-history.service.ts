import { Injectable } from '@angular/core';
import { BaseStorageHistoryService } from '@shared/utils/search-history.utils';

@Injectable({ providedIn: 'root' })
export class DownloadHistoryService extends BaseStorageHistoryService {
    constructor() {
        super('vortexdl_download_history', 5);
    }
}
