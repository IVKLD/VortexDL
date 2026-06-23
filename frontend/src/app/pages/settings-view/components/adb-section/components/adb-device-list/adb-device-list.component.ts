import {Component, inject, input, output, signal} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {MatIconButton} from '@angular/material/button';
import {MatSlideToggle} from '@angular/material/slide-toggle';
import {MatFormField} from '@angular/material/form-field';
import {MatSelect, MatOption} from '@angular/material/select';
import {FieldTree, FormField} from '@angular/forms/signals';
import {AdbDeviceSettings, StorageInfo, StorageType} from '@app/pages/settings-view/models/settings-form.model';
import {SettingsService} from '@app/pages/settings-view/settings.service';

@Component({
    selector: 'app-adb-device-list',
    imports: [MatIcon, MatIconButton, MatSlideToggle, MatFormField, FormField, MatSelect, MatOption],
    templateUrl: './adb-device-list.component.html',
    styleUrl: './adb-device-list.component.scss',
})
export class AdbDeviceListComponent {
    private readonly _settingsService = inject(SettingsService);
    private readonly _triggeredScans = new Set<string>();

    public readonly form = input.required<FieldTree<AdbDeviceSettings[]>>();
    public readonly remove = output<number>();

    protected readonly deviceStorages = signal<Record<string, StorageInfo[]>>({});

    protected loadStorages(deviceId: string) {
        this._triggeredScans.add(deviceId);
        this._settingsService.getDeviceStorages(deviceId).subscribe({
            next: (storages) => {
                this.deviceStorages.update(current => ({ ...current, [deviceId]: storages }));
            },
            error: (err) => {
                console.error(`Failed to load storages for device ${deviceId}`, err);
                this._triggeredScans.delete(deviceId);
            }
        });
    }

    protected getStorageOptions(deviceId: string, currentPath?: string): StorageInfo[] {
        const scanned = this.deviceStorages()[deviceId] || [];
        const presets: StorageInfo[] = [
            { name: 'Internal Storage', path: '/storage/emulated/0/Music', storageType: StorageType.Internal },
            { name: 'Internal Storage (Legacy)', path: '/sdcard/Music', storageType: StorageType.Internal }
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
