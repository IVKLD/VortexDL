import {ChangeDetectionStrategy, Component, inject} from '@angular/core';
import {LoadingService} from '@app/services/loading.service';
import {MatProgressBar} from '@angular/material/progress-bar';

@Component({
    selector: 'app-global-loader',
    imports: [MatProgressBar],
    templateUrl: './global-loader.html',
    styleUrl: './global-loader.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class GlobalLoader {
    protected readonly loadingService = inject(LoadingService);
}
