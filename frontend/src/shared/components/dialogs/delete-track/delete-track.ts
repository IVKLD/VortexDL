import { DIALOG_DATA } from "@angular/cdk/dialog";
import { Component, inject } from "@angular/core";
import { MatButton } from "@angular/material/button";
import { MatDialogContent, MatDialogActions, MatDialogClose } from "@angular/material/dialog";
import { MatIcon } from "@angular/material/icon";

export interface DialogDeleteTrackData {
  title: string;
}

@Component({
  selector: "app-dialog-delete-track",
  templateUrl: "./delete-track.html",
  styleUrl: "./delete-track.scss",
  imports: [MatDialogContent, MatIcon, MatDialogActions, MatDialogClose, MatButton],
})
export class DialogDeleteTrack {
  protected readonly track = inject<DialogDeleteTrackData>(DIALOG_DATA);
}
