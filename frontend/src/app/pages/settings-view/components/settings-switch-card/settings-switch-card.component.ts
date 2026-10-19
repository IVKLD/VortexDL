import { Component, input } from "@angular/core";
import { MatSlideToggle } from "@angular/material/slide-toggle";
import { FieldTree, FormField, submit } from "@angular/forms/signals";

@Component({
  selector: "app-settings-switch-card",
  imports: [MatSlideToggle, FormField],
  templateUrl: "./settings-switch-card.component.html",
  styleUrl: "./settings-switch-card.component.scss",
})
export class SettingsSwitchCardComponent {
  public readonly title = input.required<string>();
  public readonly description = input.required<string>();
  public readonly control = input.required<FieldTree<boolean>>();

  protected toggle(): void {
    const control = this.control()();
    control.value.set(!this.control()().value());
    control.markAsDirty();

    submit(this.control());
  }
}
