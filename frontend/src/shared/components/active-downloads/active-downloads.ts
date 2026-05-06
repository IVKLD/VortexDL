import {ChangeDetectionStrategy, Component, inject} from '@angular/core';
import {MatProgressBarModule} from '@angular/material/progress-bar';
import {MatButtonModule} from '@angular/material/button';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent
} from "@rx-angular/template/virtual-scrolling";
import {DownloadTrackingService} from "@app/services/download-tracking.service";
import {ActiveDownloadItemComponent} from "./components/active-download-item/active-download-item.component";
import {ActiveDownloadErrorsComponent} from "./components/active-download-errors/active-download-errors.component";

@Component({
    selector: 'app-active-downloads',
    templateUrl: './active-downloads.html',
    styleUrls: ['./active-downloads.scss'],
    imports: [
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        RxVirtualScrollViewportComponent,
        MatProgressBarModule,
        MatButtonModule,
        ActiveDownloadItemComponent,
        ActiveDownloadErrorsComponent,
    ],
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ActiveDownloadsComponent {
    protected readonly tracking = inject(DownloadTrackingService);

    clearError() {
        this.tracking.clearError();
    }

}
