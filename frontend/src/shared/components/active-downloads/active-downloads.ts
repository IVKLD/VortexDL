import {ChangeDetectionStrategy, Component, inject} from '@angular/core';
import {MatProgressBarModule} from '@angular/material/progress-bar';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent
} from "@rx-angular/template/virtual-scrolling";
import {NgOptimizedImage} from "@angular/common";
import {DownloadTrackingService} from "@app/services/download-tracking.service";

@Component({
    selector: 'app-active-downloads',
    templateUrl: './active-downloads.html',
    styleUrls: ['./active-downloads.scss'],
    imports: [RxVirtualFor, FixedSizeVirtualScrollStrategy, RxVirtualScrollViewportComponent, MatProgressBarModule, NgOptimizedImage],
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ActiveDownloadsComponent {
    protected readonly tracking = inject(DownloadTrackingService);
}
