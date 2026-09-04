import { inject, Injectable, signal } from '@angular/core';
import { SettingsTestingService } from '../../settings.service';
import { finalize } from 'rxjs';
import { FieldTree } from '@angular/forms/signals';

@Injectable()
export class SoundcloudState {
    private readonly _testingService = inject(SettingsTestingService);

    public readonly isTesting = signal(false);

    public canTest(field: FieldTree<string>): boolean {
        return !this.isTesting() && field().valid();
    }

    public testUrl(field: FieldTree<string>): void {
        if (!this.canTest(field)) return;

        this.isTesting.set(true);
        this._testingService
            .testSoundCloud(field().value())
            .pipe(finalize(() => this.isTesting.set(false)))
            .subscribe();
    }
}
