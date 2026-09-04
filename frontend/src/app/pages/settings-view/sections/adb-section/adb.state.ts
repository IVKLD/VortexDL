import { inject, Injectable, signal } from '@angular/core';
import { SettingsService } from '../../settings.service';
import { WebSocketService } from '@app/services/websocket.service';
import { finalize } from 'rxjs';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FieldTree } from '@angular/forms/signals';
import { AdbDeviceSettings, SettingsFormModel } from '../../models/settings-form.model';

@Injectable()
export class AdbState {
    private readonly _api = inject(SettingsService);
    private readonly _ws = inject(WebSocketService);

    public readonly connectedDevices = signal<string[]>([]);
    public readonly isRefreshing = signal(false);

    constructor() {
        this.refreshDevices();

        this._ws
            .connect<string[]>('/api/devices/ws')
            .pipe(takeUntilDestroyed())
            .subscribe({
                next: (devices) => this.connectedDevices.set(devices),
                error: (err) => console.error('ADB devices WebSocket error:', err),
            });
    }

    public refreshDevices(): void {
        this.isRefreshing.set(true);
        this._api
            .getAdbDevices()
            .pipe(finalize(() => this.isRefreshing.set(false)))
            .subscribe({
                next: (devices) => this.connectedDevices.set(devices),
                error: (err) => console.error('Failed to get connected ADB devices:', err),
            });
    }

    public syncDevicesWithConfig(form: FieldTree<SettingsFormModel['adb']>): void {
        const devices = this.connectedDevices();
        const current = [...form.devices().value()];
        let changed = false;

        for (const id of devices) {
            if (!current.some((d) => d.deviceId === id)) {
                current.push({
                    deviceId: id,
                    remoteMusicDir: '/storage/Music',
                    enabled: false,
                });
                changed = true;
            }
        }

        if (changed) {
            form.devices().value.set(current);
            form.devices().markAsDirty();
        }
    }

    public removeDevice(form: FieldTree<SettingsFormModel['adb']>, index: number): void {
        form.devices().value.update((current: AdbDeviceSettings[]) => current.filter((_, i) => i !== index));
        form.devices().markAsDirty();
    }
}
