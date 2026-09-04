import { Component, inject, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIcon } from '@angular/material/icon';
import { MatIconButton } from '@angular/material/button';
import { MatDivider } from '@angular/material/divider';
import { FieldTree } from '@angular/forms/signals';
import { SettingsFormModel } from '../../models/settings-form.model';
import { SettingsSwitchCardComponent } from '../../components/settings-switch-card/settings-switch-card.component';
import { AdbDeviceListComponent } from './components/adb-device-list/adb-device-list.component';
import { AdbState } from './adb.state';

@Component({
    selector: 'app-adb-section',
    imports: [
        CommonModule,
        MatIcon,
        MatIconButton,
        MatDivider,
        SettingsSwitchCardComponent,
        AdbDeviceListComponent,
    ],
    providers: [AdbState],
    templateUrl: './adb-section.component.html',
    styleUrl: './adb-section.component.scss',
})
export class AdbSectionComponent {
    protected readonly state = inject(AdbState);

    public readonly form = input.required<FieldTree<SettingsFormModel['adb']>>();

    // constructor() {
    //     effect(() => {
    //         this.state.syncDevicesWithConfig(this.form());
    //     });
    // }
}
