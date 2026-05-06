import {ChangeDetectionStrategy, Component, inject, Signal} from "@angular/core";
import {MAT_SNACK_BAR_DATA} from "@angular/material/snack-bar";
import {MatProgressSpinner} from "@angular/material/progress-spinner";

@Component({
    selector: "app-download-progress-snackbar",
    templateUrl: "./download-progress-snackbar.html",
    styleUrl: "./download-progress-snackbar.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DownloadProgressSnackbar {
}
