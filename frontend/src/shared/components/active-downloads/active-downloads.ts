import {Component, input, output} from '@angular/core';
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent,
} from '@rx-angular/template/virtual-scrolling';
import {DownloadItem} from '@app/services/download-tracking.service';
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
}
