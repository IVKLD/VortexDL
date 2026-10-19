import { inject, Injectable } from '@angular/core';
import { FieldTree } from '@angular/forms/signals';
import { SettingsTestingService } from '@app/pages/settings-view/settings.service';
import { FieldTesterState } from '@app/pages/settings-view/utils/field-tester.state';

@Injectable()
export class NetworkState extends FieldTesterState {
    private readonly _testingService = inject(SettingsTestingService);

    public testProxy(field: FieldTree<string>): void {
        this.runTest(field, (val) => this._testingService.testSingleProxy(val));
    }
}
