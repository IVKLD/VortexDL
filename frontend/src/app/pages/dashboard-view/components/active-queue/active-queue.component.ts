import {ChangeDetectionStrategy, Component, inject} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {RouterLink} from '@angular/router';
import {ActiveDownloadsComponent} from '@shared/components/active-downloads/active-downloads';
import {DownloadTrackingService} from '@app/services/download-tracking.service';

@Component({
    selector: 'app-active-queue',
    imports: [MatIcon, RouterLink, ActiveDownloadsComponent],
    templateUrl: './active-queue.component.html',
    styleUrl: './active-queue.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ActiveQueueComponent {
    public readonly tracking = inject(DownloadTrackingService);
}
