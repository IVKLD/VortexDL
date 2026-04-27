import { ChangeDetectionStrategy, Component, inject } from "@angular/core";
import {MatButton} from "@angular/material/button";
import {MatDialog} from "@angular/material/dialog";
import {DownloadDialogComponent} from "./download-dialog/download-dialog.component";

@Component({
  selector: "app-header",
    imports: [
        MatButton
    ],
  templateUrl: "./header.html",
  styleUrl: "./header.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Header {
    private readonly _dialog = inject(MatDialog);

    public openDownloadDialog() {
        this._dialog.open(DownloadDialogComponent, {
            width: '450px',
            autoFocus: 'input'
        });
    }
}
