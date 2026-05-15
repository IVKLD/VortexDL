import {ChangeDetectionStrategy, Component, computed, inject} from '@angular/core';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
} from '@rx-angular/template/virtual-scrolling';
import {DownloadStatus, DownloadTrackingService} from '@app/services/download-tracking.service';
import {ActiveDownloadItemComponent} from './components/active-download-item/active-download-item.component';
import {ActiveDownloadErrorsComponent} from './components/active-download-errors/active-download-errors.component';

@Component({
    selector: 'app-active-downloads',
    templateUrl: './active-downloads.html',
    styleUrls: ['./active-downloads.scss'],
    imports: [
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        RxVirtualScrollViewportComponent,
        ActiveDownloadItemComponent,
        ActiveDownloadErrorsComponent,
    ],
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ActiveDownloadsComponent {
    protected readonly tracking = inject(DownloadTrackingService);

    protected readonly sortedActiveDownloads = computed(() => {
        return [...this.tracking.activeDownloads()]
            .sort((a, b) => {
                if (a.status === DownloadStatus.Downloading && b.status !== DownloadStatus.Downloading) return -1;
                if (a.status !== DownloadStatus.Downloading && b.status === DownloadStatus.Downloading) return 1;
                return 0;
            });
    });

    protected clearError() {
        this.tracking.clearError();
    }
}
