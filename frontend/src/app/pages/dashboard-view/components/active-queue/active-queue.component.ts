import {ChangeDetectionStrategy, Component, inject} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {ActiveDownloadsComponent} from '@shared/components/active-downloads/active-downloads';
import {DownloadTrackingService} from '@app/services/download-tracking.service';
import {SectionHeaderComponent} from '@shared/components/section-header/section-header';

@Component({
    selector: 'app-active-queue',
    imports: [ActiveDownloadsComponent, SectionHeaderComponent, MatIcon],
    templateUrl: './active-queue.component.html',
    styleUrl: './active-queue.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ActiveQueueComponent {
    public readonly tracking = inject(DownloadTrackingService);
}
