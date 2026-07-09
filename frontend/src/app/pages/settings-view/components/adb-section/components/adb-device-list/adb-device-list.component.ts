import {Component, inject, input, output, effect, signal} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {MatIconButton} from '@angular/material/button';
import {MatSlideToggle} from '@angular/material/slide-toggle';
import {MatFormField, MatLabel} from '@angular/material/form-field';
import {MatSelect, MatOption} from '@angular/material/select';
import {FieldTree, FormField} from '@angular/forms/signals';
import {AdbDeviceSettings, StorageInfo, StorageType} from '@app/pages/settings-view/models/settings-form.model';
import {SettingsService} from '@app/pages/settings-view/settings.service';

@Component({
    selector: 'app-adb-device-list',
    imports: [MatIcon, MatIconButton, MatSlideToggle, MatFormField, MatLabel, FormField, MatSelect, MatOption],
    templateUrl: './adb-device-list.component.html',
    styleUrl: './adb-device-list.component.scss',
})
export class AdbDeviceListComponent {
    private readonly _settingsService = inject(SettingsService);

    protected readonly deviceStorages = signal<Record<string, StorageInfo[]>>({});

    public readonly form = input.required<FieldTree<AdbDeviceSettings[]>>();
    public readonly connectedDevices = input<string[]>([]);
    public readonly remove = output<number>();

    constructor() {
        effect(() => {
            for (const field of this.form()) {
                const device = field().value();
                if (this.isDeviceConnected(device.deviceId) && !this.deviceStorages()[device.deviceId]) {
                    this.loadStorages(device.deviceId);
                }
            }
        });
    }

    protected isDeviceConnected(deviceId: string): boolean {
        return this.connectedDevices().includes(deviceId);
    }

    protected loadStorages(deviceId: string) {
        this.deviceStorages.update(prev => ({...prev, [deviceId]: []}));

        this._settingsService.getDeviceStorages(deviceId).subscribe({
            next: storages => this.deviceStorages.update(prev => ({...prev, [deviceId]: storages})),
            error: err => console.error(`Failed to load storages for device ${deviceId}`, err)
        });
    }

    protected getStorageOptions(scanned: StorageInfo[] = [], currentPath?: string): StorageInfo[] {
        const presets: StorageInfo[] = [
            { name: 'Internal Storage', path: '/storage/emulated/0/Music', storageType: StorageType.Internal }
        ];

        const all = [...scanned];
        for (const preset of presets) {
            if (!all.some(s => s.path === preset.path)) {
                all.push(preset);
            }
        }

        if (currentPath && !all.some(s => s.path === currentPath)) {
            const storageType = currentPath.includes('emulated') || currentPath.includes('sdcard')
                ? StorageType.Internal
                : StorageType.SdCard;
            all.push({
                name: storageType === StorageType.SdCard ? 'SD Card' : 'Internal Storage',
                path: currentPath,
                storageType
            });
        }
        return all;
    }
}
