import {Component, computed, input, output} from '@angular/core';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
} from '@rx-angular/template/virtual-scrolling';
import {DownloadItem, DownloadStatus} from '@app/services/download-tracking.service';
import {ActiveDownloadItemComponent} from './components/active-download-item/active-download-item.component';

@Component({
    selector: 'app-active-downloads',
    templateUrl: './active-downloads.html',
    styleUrls: ['./active-downloads.scss'],
    imports: [
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        RxVirtualScrollViewportComponent,
        ActiveDownloadItemComponent,
    ],
    })
export class ActiveDownloadsComponent {
    public readonly items = input.required<DownloadItem[]>();
    public readonly remove = output<number>();

    protected readonly sortedActiveDownloads = computed(() => {
        return [...this.items()]
            .sort((a, b) => {
                if (a.status === DownloadStatus.Downloading && b.status !== DownloadStatus.Downloading) return -1;
                if (a.status !== DownloadStatus.Downloading && b.status === DownloadStatus.Downloading) return 1;
                return 0;
            });
    });
}
