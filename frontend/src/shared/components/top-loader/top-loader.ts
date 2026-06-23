import {Component, inject} from '@angular/core';
import {LoadingService} from '@app/services/loading.service';
import {MatProgressBar} from '@angular/material/progress-bar';

@Component({
    selector: 'app-top-loader',
    imports: [MatProgressBar],
    templateUrl: './top-loader.html',
    styleUrl: './top-loader.scss',
    })
export class TopLoader {
    protected readonly loadingService = inject(LoadingService);
}
