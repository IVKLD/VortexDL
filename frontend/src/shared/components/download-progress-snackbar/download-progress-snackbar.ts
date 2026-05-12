import {ChangeDetectionStrategy, Component} from '@angular/core';

@Component({
    selector: 'app-download-progress-snackbar',
    templateUrl: './download-progress-snackbar.html',
    styleUrl: './download-progress-snackbar.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DownloadProgressSnackbar {
}
