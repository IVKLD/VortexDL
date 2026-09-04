import { Component, inject, input, output, signal } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { MatIconButton } from '@angular/material/button';
import { MatSlideToggle } from '@angular/material/slide-toggle';
import { MatFormField, MatLabel } from '@angular/material/form-field';
import { MatOption, MatSelect } from '@angular/material/select';
import { FieldTree, FormField, submit } from '@angular/forms/signals';
import { AdbDeviceSettings } from '@app/pages/settings-view/models/settings-form.model';
import { StorageInfo } from '@app/pages/settings-view/models/adb-storage.model';
import { SettingsService } from "@app/pages/settings-view/settings.service";
import { AsyncPipe } from "@angular/common";
import { finalize, Observable, shareReplay } from "rxjs";

@Component({
    selector: 'app-adb-device-list',
    imports: [MatIcon, MatIconButton, MatSlideToggle, MatFormField, MatLabel, FormField, MatSelect, MatOption, AsyncPipe],
    templateUrl: './adb-device-list.component.html',
    styleUrl: './adb-device-list.component.scss',
})
export class AdbDeviceListComponent {
    private readonly settingsService = inject(SettingsService);

    public readonly form = input.required<FieldTree<AdbDeviceSettings[]>>();
    public readonly connectedDevices = input<string[]>([]);
    private readonly _storagesCache = new Map<string, Observable<StorageInfo[]>>();

    protected readonly syncingDevices = signal<Set<string>>(new Set());
    public readonly remove = output<number>();

    private toggleSyncing(deviceId: string, isSyncing: boolean) {
        this.syncingDevices.update(set => {
            const next = new Set(set);
            if (isSyncing) {
                next.add(deviceId);
            } else {
                next.delete(deviceId);
            }
            return next;
        });
    }

    protected getDeviceStorages(deviceId: string): Observable<StorageInfo[]> {
        let obs = this._storagesCache.get(deviceId);
        if (!obs) {
            obs = this.settingsService.getDeviceStorages(deviceId).pipe(shareReplay(1));
            this._storagesCache.set(deviceId, obs);
        }
        return obs;
    }

    protected isDeviceConnected(deviceId: string): boolean {
        return this.connectedDevices().includes(deviceId);
    }

    protected isSyncing(deviceId: string): boolean {
        return this.syncingDevices().has(deviceId);
    }

    protected syncDevice(deviceId: string) {
        this.toggleSyncing(deviceId, true);

        this.settingsService.syncDevice(deviceId).pipe(
            finalize(() => this.toggleSyncing(deviceId, false))
        ).subscribe();
    }

    protected triggerSave<T>(field: FieldTree<T>) {
        submit(field);
    }
}
