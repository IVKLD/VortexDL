import {ChangeDetectionStrategy, Component, input, output} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {MatIconButton} from '@angular/material/button';
import {MatSlideToggle} from '@angular/material/slide-toggle';
import {MatFormField, MatInput} from '@angular/material/input';
import {FieldTree, FormField} from '@angular/forms/signals';
import {AdbDeviceSettings} from '@app/pages/settings-view/models/settings-form.model';

@Component({
    selector: 'app-adb-device-list',
    imports: [MatIcon, MatIconButton, MatSlideToggle, MatFormField, MatInput, FormField],
    templateUrl: './adb-device-list.component.html',
    styleUrl: './adb-device-list.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AdbDeviceListComponent {
    public readonly form = input.required<FieldTree<AdbDeviceSettings[]>>();
    public readonly remove = output<number>();
    public readonly toggleDevice = output<number>();
}
