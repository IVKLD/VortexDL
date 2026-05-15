import { ChangeDetectionStrategy, Component, inject, output, signal } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatIconButton } from '@angular/material/button';
import { SettingsService } from '@app/pages/settings-view/settings.service';
import { finalize } from 'rxjs';

@Component({
    selector: 'app-adb-discovery',
    imports: [MatIcon, MatIconButton],
    templateUrl: './adb-discovery.component.html',
    styleUrl: './adb-discovery.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AdbDiscoveryComponent {
    private readonly _api = inject(SettingsService);

    public readonly selectDevice = output<string>();
    protected readonly availableDevices = signal<string[]>([]);
    protected readonly isRefreshing = signal(false);

    constructor() {
        this.refreshDevices();
    }

    protected refreshDevices() {
        this.isRefreshing.set(true);
        this._api.getAdbDevices()
            .pipe(finalize(() => this.isRefreshing.set(false)))
            .subscribe({
                next: (devices) => this.availableDevices.set(devices)
            });
    }
}
