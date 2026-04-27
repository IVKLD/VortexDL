import { inject, Injectable } from '@angular/core';
import { MatSnackBar, MatSnackBarRef, TextOnlySnackBar } from '@angular/material/snack-bar';

@Injectable({
  providedIn: 'root'
})
export class NotificationService {
  private readonly _snackBar = inject(MatSnackBar);
  private _persistentRef?: MatSnackBarRef<TextOnlySnackBar>;

  public showSuccess(message: string): void {
    this._snackBar.open(message, 'Close', {
      duration: 3000,
      horizontalPosition: 'right',
      verticalPosition: 'bottom',
      panelClass: ['success-snackbar']
    });
  }

  public showError(message: string): void {
    this._snackBar.open(message, 'Close', {
      duration: 5000,
      horizontalPosition: 'right',
      verticalPosition: 'bottom',
      panelClass: ['error-snackbar']
    });
  }

  public showInfo(message: string): void {
    this._snackBar.open(message, 'Close', {
      duration: 3000,
      horizontalPosition: 'right',
      verticalPosition: 'bottom'
    });
  }

  public showStatus(message: string): void {
    this._persistentRef = this._snackBar.open(message, undefined, {
      horizontalPosition: 'right',
      verticalPosition: 'bottom',
      panelClass: ['status-snackbar']
    });
  }

  public dismissStatus(): void {
    if (this._persistentRef) {
      this._persistentRef.dismiss();
      this._persistentRef = undefined;
    }
  }
}
