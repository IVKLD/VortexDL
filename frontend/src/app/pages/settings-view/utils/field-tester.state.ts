import { signal } from '@angular/core';
import { FieldTree } from '@angular/forms/signals';
import { Observable, finalize } from 'rxjs';

export abstract class FieldTesterState {
    public readonly isTesting = signal(false);

    protected runTest(field: FieldTree<string>, testFn: (value: string) => Observable<unknown>): void {
        if (!this.canTest(field)) return;

        this.isTesting.set(true);
        testFn(field().value())
            .pipe(finalize(() => this.isTesting.set(false)))
            .subscribe();
    }

    public canTest(field: FieldTree<string>): boolean {
        return !this.isTesting() && field().valid();
    }
}
