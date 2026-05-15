import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { FieldTree, FormField } from '@angular/forms/signals';
import { MatSlideToggle } from '@angular/material/slide-toggle';
import { AdbDeviceSettings, AdbSettings } from '@app/pages/settings-view/models/settings-form.model';
import { MatDivider } from '@angular/material/divider';

import { AdbDiscoveryComponent } from './components/adb-discovery/adb-discovery.component';
import { AdbDeviceListComponent } from './components/adb-device-list/adb-device-list.component';

@Component({
    selector: 'app-adb-section',
    imports: [MatIcon, MatSlideToggle, FormField, AdbDiscoveryComponent, AdbDeviceListComponent, MatDivider],
    templateUrl: './adb-section.component.html',
    styleUrl: './adb-section.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AdbSectionComponent {
    public readonly form = input.required<FieldTree<AdbSettings>>();

    protected useDevice(id: string) {
        const current = this.form().devices().value();
        if (current.some(d => d.deviceId === id)) return;

        const device: AdbDeviceSettings = {
            deviceId: id,
            remoteMusicDir: '/storage/Music',
            enabled: true,
        };

        this.form().devices().value.set([...current, device]);
    }

    protected removeDevice(index: number) {
        this.form().devices().value.update(current => current.filter((_, i) => i !== index));
    }

    protected toggleDevice(index: number) {
        this.form().devices().value.update(current =>
            current.map((device, i) => i === index ? { ...device, enabled: !device.enabled } : device)
        );
    }
}
