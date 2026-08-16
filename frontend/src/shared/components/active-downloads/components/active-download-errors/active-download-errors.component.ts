import {Component, input, OnInit, output} from '@angular/core';
import {MatIcon} from '@angular/material/icon';
import {MatDivider} from "@angular/material/list";
import {MatTooltip} from "@angular/material/tooltip";
import {MatButton} from "@angular/material/button";

@Component({
    selector: 'app-active-download-errors',
    imports: [MatIcon, MatDivider, MatTooltip, MatButton],
    templateUrl: './active-download-errors.component.html',
    styleUrl: './active-download-errors.component.scss',
})
export class ActiveDownloadErrorsComponent implements OnInit {
    public readonly errors = input.required<string[]>();

    public readonly clear = output<void>();
    public readonly openViewErrorsDialog = output<void>();
    public readonly autoClose = output<void>();

    ngOnInit() {
        setTimeout(() => this.autoClose.emit(), 10 * 1000)
    }
}
