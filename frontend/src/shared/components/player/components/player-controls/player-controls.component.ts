import { Component, output } from "@angular/core";
import { MatIconButton } from "@angular/material/button";
import { MatIcon } from "@angular/material/icon";
import { MatTooltip } from "@angular/material/tooltip";
import { PlayerDialComponent } from "../player-dial/player-dial.component";

@Component({
  selector: "app-player-controls",
  imports: [MatIcon, MatIconButton, MatTooltip, PlayerDialComponent],
  templateUrl: "./player-controls.component.html",
  styleUrl: "./player-controls.component.scss",
})
export class PlayerControlsComponent {
  public readonly previous = output<void>();
  public readonly next = output<void>();
}
