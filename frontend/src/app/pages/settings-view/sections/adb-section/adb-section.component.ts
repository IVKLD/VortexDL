import { Component, input, output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatIconButton } from '@angular/material/button';
import { MatDivider } from '@angular/material/divider';
import { FieldTree } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SettingsSwitchCardComponent } from '../../components/settings-switch-card/settings-switch-card.component';
import { AdbDeviceListComponent } from './components/adb-device-list/adb-device-list.component';

@Component({
    selector: 'app-adb-section',
    imports: [
        CommonModule,
        MatIcon,
        MatIconButton,
        MatDivider,
        SettingsSwitchCardComponent,
        AdbDeviceListComponent
    ],
    templateUrl: './adb-section.component.html',
    styleUrl: './adb-section.component.scss'
})
export class AdbSectionComponent {
    public readonly form = input.required<FieldTree<SettingsFormModel['adb']>>();
    public readonly connectedDevices = input.required<string[]>();
    public readonly isRefreshing = input.required<boolean>();

    public readonly refresh = output<void>();
    public readonly removeDevice = output<number>();
}
