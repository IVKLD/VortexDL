import {Component, inject} from '@angular/core';
import {MAT_DIALOG_DATA, MatDialogActions, MatDialogClose, MatDialogContent, MatDialogTitle} from "@angular/material/dialog";
import {MatButton} from "@angular/material/button";
import {MatIcon} from "@angular/material/icon";

export enum ErrorsDialogResult {
    Clear = 'clear'
}

@Component({
    selector: 'app-errors-dialog',
    imports: [
        MatDialogTitle,
        MatDialogContent,
        MatDialogActions,
        MatDialogClose,
        MatButton,
        MatIcon
    ],
    templateUrl: './errors-dialog.component.html',
    styleUrl: './errors-dialog.component.scss'
})
export class ErrorsDialogComponent {
    protected readonly errors = inject<string[]>(MAT_DIALOG_DATA);
    protected readonly ErrorsDialogResult = ErrorsDialogResult;
}